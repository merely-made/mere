# Link model: assertions are the truth, links are readings

**Date:** 2026-10-15
**Status:** design, no code changed. Mark ruled L1–L4 on 2026-10-15 (§4).
Phases P1–P6 (§8) are unscheduled, and the open rulings in §9 remain open.

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
when and on what basis each held. Everything else is a reading of those
assertions, computed and never stored: the link that gathers them, the stroke
that draws it, its weight, its bend, the force it exerts and the timeline it
opens into. A concept stays only if it records an assertion, states what one
means, or reads assertions for a view. Any other concept is retired.

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
   arrangement of an opened link.* It needs data that does not exist today:
   first-assertion time, timed withdrawal and a basis (§6). Most members are
   timeless, so the timeline carries an **undated lane** (§7.2).
3. **The bend shows force, freed because fanning stops at rest.** *Kept.* With
   one stroke per link (ruling L4), geometry is free. Bend and weight read
   from one per-link spring whose stiffness splits exactly across the link's
   assertions. That split is what makes per-assertion force attribution
   possible for the first time (§7.3).
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
  | `id` | opaque, device-safe, stable across snapshot, replay and sync | `statement_id` (semantic only) |
  | `subject`, `object` | direction is kept; the link reading is unordered | pair bucket direction |
  | `predicate` | meaning, as an IRI; recognized predicates carry behavior | `predicate` plus sub-kind sets |
  | `scope` | named graph | `graph_scope` |
  | `asserter` | attributed source, else the journal `Author` (ruling 5) | `provenance_iri` |
  | `first_asserted_ms` | set when the assertion is minted, never overwritten | missing |
  | `last_asserted_ms` | updated on reassertion by the same asserter | `asserted_at_ms` (often `None`) |
  | `withdrawn` | `{ at_ms, by }` once withdrawn; the record remains until compaction | missing (hard delete) |
  | `basis` | why: `note`, `method`, `evidence`, `occurrences` (§6.2) | `label` only |

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
  or minted. When a Resource link is drawn between surfaces it is one link
  with many appearances, and it keeps its Resource origin (TERMINOLOGY). If
  a pair ever needs an identity of its own, for comments or shared review, it
  is **promoted**: it is frozen into a Resource the way a selection is frozen
  (ruling 8), with assertion copies at a revision. Under the resource-leaf
  draft, a selector alone mints no entity.
- **edge**: unchanged. A relation across strata, today the zero-hop shown
  binding. This design adds no new edge kind.
- **strength** (view rule): a per-view function from a link's held
  assertions to one number. Its default is the sum over held assertions of
  each predicate's declared force contribution. Stroke weight and spring
  stiffness both read it, and it splits exactly across assertions. Strength
  is a setting, not truth (audit thread 4: "the strength projection probably
  wants to be settings").

Traversal joins this set rather than standing beside it. The pair's
navigation history is one assertion per direction: predicate
`mere:traversed`, the persona as asserter, and a span from the first to the
last navigation. Its dated events are that assertion's **occurrences**, which
are today's `TraversalData` events and `EdgeMetrics` rollup moved under the
assertion. Woodshed's practice evidence has the same shape: an evidence
assertion whose occurrences are dated visits. This answers the §7
"Occurrences or events" candidate for pair relations without a new stratum.
An occurrence is dated support inside an assertion, not an entity.

## 4. Rulings (Mark, 2026-10-15)

Asked as four questions. Mark took the recommended option each time.

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
  opening the link. All three fan implementations retire.

## 5. Retire, subsume, keep

**Retire** means the concept is deleted. **Subsume** means another concept
covers its job. **Keep** means it stays, possibly re-expressed.

| Concept (where) | Verdict | Becomes |
|---|---|---|
| `EdgeFamily` (6; `edge_taxonomy.rs`) | Subsume | A declared predicate property (the policy group). No longer a storage shape. The stance doc's curated lens survives as registry vocabulary. |
| Sub-kinds (`SemanticSubKind` and the four others; 43 recognized kinds) | Subsume | Recognized predicates. Each enum is the recognized-predicate table for its family. |
| `EdgePayload` six sidecars (`edge_payload.rs`:46-53) | Retire (P6) | One assertion list per pair bucket. The type name stays as the bucket, under the `edge` rule. |
| `SemanticData` compatibility aggregates (`sub_kinds`, `label`, `predicate`) | Retire | Nothing; readers iterate assertions. |
| `ArrangementData` / `ContainmentData` / `ImportedData` / `ProvenanceData` sub-kind sets | Retire (P6) | Assertions with id, asserter, span and basis. |
| `TraversalData` + `EdgeMetrics` | Subsume | The occurrences and rollup of a `mere:traversed` assertion. |
| `EdgeKind`: graphshell 4-variant tag set, kernel index | Already retired 2026-05-13 | Delete the surviving doc comments when touched (`mod.rs`:154, `edge_payload.rs`:14). |
| VGCP `EdgeKind` | Already "not current direction" | No action. |
| tabard `EdgeStyleRegistry`, `EdgeStyleKey`, `ThemeEdgeKindToken` (`theme/edge_style.rs`) | Retire | Stratum stroke cue (ruled 2026-10-13) plus a per-predicate-family style token in the same theme. It is a fifth, test-only style taxonomy with no Provenance family. |
| `RelationKind` (read classifier) | Subsume | The recognized predicate. |
| `RelationSelector` (filter and retract discriminant) | Subsume | A predicate filter: one predicate, a family group or a set. Exact retract uses the assertion id. |
| `EdgeAssertion` (write contract) | Keep, simplified | `assert(spec)` and `withdraw(id)`, the only two writes. |
| `RelationKey` (pair-bucket handle) | Keep | Internal. Not a link id, not an assertion id. |
| `StatementId` | Keep | The assertion id, the only member identity. |
| `decay_progress` (written, then discarded: `edge_ops.rs`:104, snapshot constant 0.0) | Retire | Strength (view rule) and lifecycle (predicate). |
| `RelationOrigin` (deferred 2026-05-11, never built); Woodshed `RelationAuthority` {Catalog, Computed, Evidence}; forme member `Provenance`; "explicit / computed / inferred" | Subsume | One derived **authority reading**: `computed` when `basis.method` is set, `evidence` when evidence or occurrences are set, otherwise `explicit`. The asserter tells whose. No stored authority field. |
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
| Multiplicity as thickness (2026-06-24) and as N spring slots (`seiche_bridge.rs`) | Subsume | Strength. |
| Visual edge-collapse "LOD setting" (petgraph plan; swatch design §7, never built as a switch) | Subsume | Rest stroke and opened link. |
| Swatch design §7 "the addressable edge is a cell" | Retire | The addressable unit is the assertion; the drawable unit is the link. |
| Hide-only vs retract by authorship | Keep | Withdraw is allowed to the asserter, per the predicate's retractability. Hide is curation. |
| pandect `HiddenRelationRecord {from, to, relation_tag}` (`view_intent_store.rs`) | Keep, re-keyed | Hides one assertion by id, a predicate filter on a link, or a whole link. A family tag cannot tell two assertions apart. |
| sceno `RoutedRelation` | Keep | Gains a source reference to the link key, so the scene path keeps identity (`scene_out.rs` collapses it today). |
| `VisualRelationKind` / `StrokeCue` (query view) | Keep | The stratum cue of the rest stroke. |
| scenomise `DisclosedRelationship` with `RelationshipProvenance` | Keep | The disclosure shape of one assertion from a host. Its fields align with `basis`: method, method version, provider, evidence. |
| Woodshed `MaterialRelation` | Keep, disclosed | A domain record disclosed as an assertion. `method` is the Woodshed computation and version. Its 0..100 weight is a domain value the strength function may read. |
| `EdgePath` (`numen/edge_path.rs`) | Keep | Gains its first consumer: the link bend (§7.3). |
| Field couplings | Keep | Node-targeted, unchanged. |
| Nested graph | Keep | Node-borne. Not used for links; promotion uses the freeze pattern instead. |
| Swatch | Keep | Gains a **link** scope (§7.2). |
| Timeline arrangement (sceno `score.rs`) | Keep | The default arrangement of an opened link. |
| "Timeline" × 6 | Clarify | In this doc it means only the arrangement family. |
| Linked-data first-arc surface projection (`linked-data/src/lib.rs`:360) | Retire | Reads every arc through `edges_between_undirected`, as the kernel already does. |

**Count.** Of 47 rows, 23 are retired or subsumed and two were already
retired. They include three fans, three bundle
keys, four identity schemes, two Link Cards, the style registry, the
authority enums and the dead decay field. Added: three fields
(`first_asserted_ms`, `withdrawn`, `basis`), one read model that replaces four,
and one swatch scope. No new stratum, entity, edge kind or stored type.

## 6. Time and why

### 6.1 Time

An assertion carries **record time**: when its asserter said it, when they
last confirmed it and when it was withdrawn. The span is
`[first_asserted_ms, withdrawn.at_ms or open)`. Reassertion by the same
asserter after a withdrawal mints a **new** assertion, because dedup matches
held records only, so a pair's history is a sequence of closed intervals,
never a revived one. Asserter-claimed **valid time** ("these were related from
2010 to 2015") is not a field. It is open ruling O1.

Clock rule: the caller's time is used when the caller supplies one: Knot's
signed header, an ingested `prov:generatedAtTime`, a sync event. Otherwise the
kernel stamps wall clock at write. A record with no time is undated (legacy
data, or a source that gives none). It is shown in the undated lane and never
as 1970.

### 6.2 Why: `basis`

| Field | Holds | Source today |
|---|---|---|
| `note` | free-text reason | `label` |
| `method` | `{ iri, version }` of the rule, script, engine or model that computed it | journal `Author` (dropped from the statement); scenomise `method`/`method_version` |
| `evidence` | IRIs of supporting resources, captures or other assertions | Knot `evidence`; scenomise `evidence` refs |
| `occurrences` | dated events plus rollup (count, first, last) for event-backed predicates | `TraversalData` + `EdgeMetrics`; Woodshed `Engagement` |

There is no stored confidence, weight or authority field. The authority
reading (§5) and strength (§3) are both derived. Knot's `qualification` maps to
`note` until a consumer needs it separately.

### 6.3 Minimal write-path changes

1. **Stamp every route.** `apply.rs` (`AssertRelation`, replay deltas),
   `edge_ops.rs` (`assert_relation`, predicate setter) and graphshell personal
   sync stop passing `None`. P1's existing test that enumerates production
   writers is extended to assert that each writes a time.
2. **First and last.** `assert_statement` sets `first_asserted_ms` on mint and
   leaves it alone on reassertion. Today's `asserted_at_ms` becomes the last
   time, and its persisted name stays. Legacy records load with `first = last =`
   the stored value, or undated.
3. **Withdraw, don't delete.** `retract_statement` sets
   `withdrawn { at_ms, by }` instead of removing (`assertion_write.rs`:799-846;
   `edge_data.rs`). Held readers filter it out. The bucket is removed only when
   it holds no record at all. Graphshell's `RetractAssertion` event gains a
   time. Compaction is per predicate lifecycle (statement kernel brief) or by
   accepted checkpoint for shared assertions.
4. **Basis.** Persist `note`, `method`, `evidence` and `occurrences` in
   `PersistedSemanticStatement`. Ingest and export round-trip them.
5. **RDF.** On the reifier: `prov:generatedAtTime` = first;
   `mere:lastAssertedAt` = last; `prov:invalidatedAtTime` and
   `prov:wasInvalidatedBy` for a withdrawal; `prov:wasGeneratedBy` for the
   method; `prov:wasDerivedFrom` for evidence. Ingest lifts all of them, not
   only today's three (`ingest.rs`:507-531). Whether default SPARQL sees
   withdrawn assertions is open ruling O2.
6. **All families (P6).** The four sidecar families write assertions through
   the same `assert` and `withdraw`. Their compatibility reads become predicate
   filters.

## 7. Consequences, not layers

### 7.1 The link

`link(stratum, a, b)` reads every arc in both directions through
`edges_between_undirected` (`edge_ops.rs`:644), plus lifted appearances
through `projected_relations` (`query.rs`). It yields assertions grouped by
nothing else. Direction, predicate, asserter and family are attributes of
members, not levels. There is one implementation, which every consumer reads:
Graphshell, roster, pictograph, cambium and Woodshed. Hosts outside the kernel
(Woodshed and Knot via scenomise) disclose assertions in the same shape.
Where that shared shape lives is open ruling O3.

### 7.2 The opened link, which is the "nested link"

Opening a link (select, or zoom past a threshold) shows a **swatch at link
scope**. That is the existing swatch with one new scope value, compiled to a
scene in the existing Timeline family:

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
assertion id. Its accessible alternate is the same list, read in time order
with the undated members last. Nothing in it is stored. It is a projection
of the link, so it inherits the swatch's representation-never-identity rule.

### 7.3 The force-carrying bend

At rest a link is one stroke and one spring. Stiffness is the link's
strength, which splits exactly across held assertions (§3), so a force
measured on the spring is attributed to each assertion in proportion to its
share. The dynamics inspector's "Per-assertion force attribution … not
available" (`dynamics_inspection.rs`:138) closes. It does not close by
measuring per assertion; it closes because the model makes the split
definitional. This replaces N spring slots per visible row (`seiche_bridge.rs`).

The bend reads the spring's state. A link stretched past its rest length runs
straight and tightens. A slack link bows. Lateral load, from fields or a
neighbor, deflects it. Width shows strength. The bend uses `EdgePath::Spline
{ tension }`, its first consumer. Hiding a member lowers strength in that
instance only, so the spring relaxes there and nowhere else (swatch design §8,
unchanged). "Relationship membership alone does not install a force"
(dynamics grammar plan) still holds: a predicate's force contribution is
declared, and it can be zero.

### 7.4 What falls away

The fan-or-collapse switch the swatch design promised but never built is no
longer a choice. The swatch's §11 question about per-cell fan geometry ("how
many parallel lines … stay legible") dissolves, because no view fans.
Multiplicity stops meaning three different things (thickness, spring slots,
cell count). The audit's thread 2, edge reification, is answered: assertions
are addressable, and the pair is their reading. Thread 4, unified strength,
is answered as a view rule.

## 8. Phases

Each phase lands green before the next. Each has a first proof consumer, in
the order the survey ranked them.

- **P1. Time and withdrawal (kernel, linked-data).** Implements §6.3 items 1
  to 5 for semantic assertions. Proof: **Graphshell's link inspector** lists
  a pair's assertions sorted by first time, with withdrawn spans and the
  undated ones last. A Knot assertion and retraction round-trip with their
  times. Done when no writer leaves time empty, reassertion keeps the first
  time, and withdraw-then-reassert yields two intervals.
- **P2. One link reading.** `link(stratum, a, b)` in one place. Graphshell
  `LinkInspection`, roster `LinkCard`/`LinkBundle` and pictograph
  `PairInputs`/`EdgeCell` read it, and their own groupings are deleted. Proof:
  Graphshell inspector unchanged in output, roster card gains asserter and
  time. Includes the linked-data every-arc fix.
- **P3. Opened link.** Swatch link scope plus Timeline with lanes and an
  undated lane. Proof: Graphshell, then **Woodshed** (selection,
  explanation, method provenance, P4f audition per assertion), then **Knot**
  (authored time and retraction).
- **P4. Rest stroke, fan retired.** One stroke per link in pictograph,
  cambium `graph_canvas` and the scene path (`RoutedRelation` gains its link
  key). Hit-test resolves the link; opening resolves a member. pandect hide
  is re-keyed. Proof: **Woodshed's relationship editor**, the surface that
  started this lane, then the Graphshell canvas.
- **P5. Force-carrying bend.** One spring per link with split stiffness, a
  per-link force readout from seiche, the `EdgePath` spline bend, and
  inspector attribution per assertion. Proof: Graphshell dynamics inspector.
- **P6. All families as assertions.** The four sidecar families become
  assertion records, `EdgePayload` collapses to one list, `RelationKind` and
  `RelationSelector` become predicate filters, and the tabard style registry
  retires. Persistence migration loads old snapshots, with asserters from
  journal replay as in ruling 9. Proof: roster and pandect hide one
  containment assertion of two on a pair.

## 9. Open rulings for Mark

- **O1. Valid time.** Should an asserter be able to claim when a relation held
  in the world, beyond record time? Recommended: not yet. When a consumer
  needs it, add it as one optional `basis` interval, never as the timeline's
  default axis.
- **O2. History in SPARQL.** Should default SPARQL and export see only held
  assertions, with withdrawn ones behind an explicit history scope?
  Recommended: held only by default.
- **O3. Where the shared link shape lives.** The kernel cannot depend on
  scenomise, and Woodshed reaches Mere through chartulary and scenomise.
  Recommended: the portable assertion-disclosure and link-key shape in
  `scenograph`, beside `DisclosedRelationship`'s fields, with the kernel
  adapting its native assertions into it. Decide before P2.
- **O4. Chartulary convergence.** Chartulary models one edge per relation
  with `EdgeId(writer, counter)`, which is already an assertion with an id,
  and the kernel models a bucket of statements. Should chartulary relations
  gain asserter, span and basis so Woodshed's links have time? Recommended:
  yes, in P3, with chartulary's `EdgeId` as the assertion id.
- **O5. Strength defaults.** Each recognized predicate's force contribution,
  and whether traversal counts or recency feed it. Recommended: settle with
  the dynamics grammar plan's G-phases rather than here.
- **O6. Open threshold and lanes.** When does a link open on zoom, and does
  the Timeline family need a second (lane) axis? The Woodshed hocket survey
  brief (`woodshed/design_docs/2026-10-14_dsp_graphshell_hocket_survey_brief.md`:210)
  flags the same one-axis limit. Recommended: lanes as stacked rows inside
  Timeline, with no new family.
- **O7. Promotion.** Is freezing a link into a Resource, for comments and
  shared review, wanted now? Recommended: no; record it as the escape hatch
  only.
- **O8. Compaction windows.** How long a locally withdrawn assertion stays
  before its predicate's lifecycle compacts it. Recommended: content
  predicates keep withdrawals until a checkpoint; session and arrangement
  predicates drop at session end.

## 10. Vocabulary

TERMINOLOGY gains **assertion** and an L1 note on **link** (a derived reading
keyed by stratum and unordered endpoints, never stored). "Linking factor" is
recorded as Mark's word for an assertion. **Cell** and the roster sense of
**bundle** retire from design language. Code identifiers keep their names
until touched, under the `edge` rule.
