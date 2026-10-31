# Graph semantics plan: assertions, resources, strata, residency and the resident host

**Status:** 2026-10-28: cut from `2026-10-04_graph_semantics_plan.md`,
`2026-10-09_graph_semantics_followons_plan.md`, `2026-10-09_resident_host_implementation_plan.md`,
`2026-06-05_node_navigation_lineage_wiring_plan.md`, `2026-07-08_generic_graph_substrate_plan.md`
and lane L1 of `2026-10-20_ownership_repair_plan.md`, archived in `archive_docs/2026-10-28/`. The
bounded P1-P5 lane is published on main and rulings 1-46 stand; the resident host's Tasks 1-3 and
the host half of Task 4 are on main; the substrate's G0-G2 landed. Open: consumer repinning, the
rest of the resident host, scoped transactions and collection, the strata draft, lineage on a live
host, the substrate's G3-G5 restatements, and the rest of lane L1.

**For:** what a mere's graph records and how much of it is present: assertions, the resource and
surface strata, the pending-link index and coverage, saved queries and frozen selections, partial
residency, the resident host that presents and edits a partial graph, navigation lineage, and the
chartulary substrate beneath. It opened with Mark's four questions of 2026-10-04 (what a link
records, what makes two things the same, what a saved query becomes, how much of the graph must be
present), was bounded by his "Bound it." of 2026-10-07, and took on the resident host when he
approved its port on 2026-10-15.
Not here: link reading, assertion span, withdrawal, basis and promoted links
(`2026-10-28_link_model_plan.md`); the Graphshell reference host's phases
(`2026-10-28_graphshell_host_plan.md`); the remote overmap crate (`2026-10-28_stack_lift_plan.md`);
the gloss host and swatch visibility stack that lineage P4 needs
(`2026-10-28_graph_view_curation_plan.md`); ownership repair lanes L2 (projections), L3 (saved
versus live content), L5 (transport admission) and L6 (receipts), the routing of the audit's
unrepaired findings and the start order of L2-L5 (`2026-10-28_source_audit_repairs_plan.md`); L4,
custody lifetimes (`2026-10-28_custody_plan.md`); L6's djinn-testkit slice, Q052 and Q054
(`2026-10-28_djinn_test_harness_plan.md`). Vendored GPU patches go to the upstream candidates
ledger, outside every plan (Mark).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this plan (DOC_POLICY §8).
Evidence that matters is a test in the repository or a line in Findings.

## Constraints

- Rulings 1-46 are the contract, in Mark's words at
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:2293-2684` (placement table
  `:2753-2776`): separate assertions per (sub-kind, predicate, scope, asserter) (1, 5, 9); content
  claims on one resource per canonical URL, a surface recording the resource it shows (2, 6, 22,
  29); "resource" and "surface", `ResourceNode` and `SurfaceNode`, `Node` and `NodeKey` kept for
  surfaces (12, 13, 16, 17, 19); placement per predicate by nature, kept with the mere (10, 14, 26,
  34-36); tags are resources, classification lives on them (15, 20, 21, 23, 24); migration replays
  the journal first, composition merges resources by canonical identity, the caller-qualified legacy
  adapter stays until a durable placement profile exists (11, 18, 27, 37, 40-42); shown bindings
  cost zero hops, JSON-LD uses classic reification, sealed-wrapper transactions are in scope (25,
  28, 30-33, 38, 39, 43, 44); share the spec, freeze on demand (3, 8, 46); partial residency with a
  coverage note on every result and a rebuildable pending index, session-only by default (4, 7, 45).
- The "Bound it." boundary (`:2687-2750`): no numbered checkpoint is added automatically; an
  optional discovery is deferred with evidence; a mandatory contradiction with a ruling or
  done-condition stops the work with one scope-exception report; nothing from another lane enlarges
  the bound without Mark's amendment; an API is never broken silently.
- Follow-on lane
  (`archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:14-32,220-232`): commits
  stay local until Mark reviews a qualified candidate; each shippable phase has its own checkpoint;
  library tests, host behaviour, remote availability and memory probes are separate claims.
- Resident host
  (`archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:55-61,80-92`): Ruling 1,
  live edits ride `transact_change` and refuse when storage refuses, so the retained batch,
  `MixedCommit`, `AppliedButUnstored` and `Busy` pause are dropped (the lane's Global Constraints
  bullet "retry the retained batch, never the command" predates it); Ruling 2, resident edits are
  admitted only through the per-session host map, a Directory-owning cache only if a second writer
  is shown possible; Ruling 4, the double full-graph encode is recorded, not fixed. Full stays the
  compatibility setting; `ResidentGraph` is immutable; no multiwriter lock service, authority epochs
  or receipt log; Keep stays per-view and every pane's focus counts without a durable pin; no
  generation collection yet; process memory is reported apart from reader savings; backend work is
  limited to IndexedDB lifetime admission.
- Ownership repair
  (`2026-10-20_ownership_repair_plan.md:55-72,184-196,216-404`): only Mark
  numbers rulings; a lane stops at a fork and brings the code evidence; one worktree and build
  directory per lane; sealed audit registers are never edited (dated successor receipts supersede
  them). L1 rulings given: the stored node address wins on load; insert-if-missing live; a deleted
  owner's creator inherits its children, else they clear; malformed facet JSON is refused at checked
  load and in migration; the kernel `store` feature is removed; one handle is one row; edge replays
  join that rule; a checkpoint the journal cannot confirm is replayed past (a retained-translation
  session still refusing a difference as Corrupt); no entry format version is stored.

The ownership repair plan is not archived: its L1 round was live on 2026-10-28 in another session, so this plan cites it at its live path and its line numbers may drift.

## Phases

### A. Supplier qualification and consumer repin
In isolated successors of Turnstone, Knot and Redshank, repin every Mere Git row in the consumed
workspaces (patch rows included; unrelated Genet and engine pins and Woodshed root rows stay),
preserving unpublished ancestors and qualifying providers before Turnstone chooses their hashes. A
local candidate, or a process-only local Git route to an unpublished supplier, proves nothing about
remote availability. Held: the typed resource capture protocol's (ruling 25) production patch,
dependency integration and main review, and Turnstone's prepared reader adoption on its old pin.
- Done when: sibling scoped tests and all-target checks pass; Turnstone default and Piccolo checks
  pass; metadata resolves one coherent Mere source; capture admission, identity, persistence and
  refusal, fork and journal-origin controls stay intact; suppliers and providers publish before
  consumers and a clean remote resolution is verified.
- Needs: qualified immutable Knot and Redshank hashes before Turnstone changes provider references;
  Mark's approval to push and merge.
- Rulings: 25
- From: `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:45-48,75-88,282-283`;
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:1993-2000,2889-2892`

### B. RDF retirement closure and the query tails
Remove the old Oxigraph Store module and optional query dependency in `crates/graph/linked-data`,
keeping the borrowed-versus-materialized spareval battery as the parity oracle, SEP-0002, SEP-0006,
calendar support and JSON-LD RDF 1.2 kept explicitly, with three exact-output controls. Review
conclusions, not a ruling: ExampleOf stays MereOnly; Summarizes keeps its approximate `cito:cites`,
and `cito:describes` changes the exported quad so needs its own vocabulary ruling. Completion is
recorded in Progress here, the deferred-tails plan the source names being archived by this cut.
Deferred, unscheduled: CONSTRUCT and DESCRIBE into a subgraph, a shared `mapping` module, RDF-star
edge metadata beyond the landed reifiers, a results pane, semantic-surface JSON-LD in view, a
synced-mirror store, federation, `UPDATE` and HDT, term dictionaries, and the interned-slotmap
kernel (gated on the 2026-07-06 footprint measurement).
- Done when: feature negatives turn green after explicit retention; the full linked-query and
  Mere-query suites, the feature-free linked check, the locked workspace check and both query wasm
  checks pass; one fresh source-qualified review clears the retirement; the parity oracle remains.
- Needs: Mark's approval to publish.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:115-143`;
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:2720-2729,4042-4044`;
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:25-35,56-59,1028-1039,1045-1054`

### C. Resident host, Tasks 4 remainder, 5 and 6: roots, refresh, coverage
Task 4: Djinn attaches and releases pane roots through the endpoint lifetime (`MereEndpoint` has no
drop hook), each admitted endpoint gets its own `ViewKey`, and `MereShared::host` turns publication
on once the persisted settings slot exists. Task 5: `refresh_if_changed` reads only the head; a
candidate and install split with `RefreshOutcome`; `RefreshGate` and `RefreshToken` give single
flight and notice coalescing; hosts drive checks from foreground, event, idle and publication
boundaries. Task 6: `PortableCoverageV1` on `ProjectionSnapshot` and `MountedScene`, defaultable on
the wire, with Djinn's coverage-only notice, demand and settings actions; missing old-wire metadata
does not prove completeness.
- Done when: Task 4, scoped roots demonstrably retain less reader data, Full clears Residency limits
  while other coverage layers stay, two panes retain their union and closing one releases only its
  own roots. Task 5, unchanged polling reads only head, failed or stale candidates never replace the
  coherent view, bursts coalesce, foreground catches missed publications, native and Firefox
  IndexedDB drive the same transitions (automatic checks promise nothing while a tab is suspended).
  Task 6, endpoint clients receive and retain the five coverage meanings, including context-only
  changes.
- Needs: fork F3 ruled; Knot repinned to the 0.0.5 scene crates so Djinn `resident_mere` compiles;
  the browser pump (`web_resident`) location, decided with D.
- Rulings: resident 1, 2
- From: `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:20-48,63-75,404-672`;
  `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:63-64,90-113,176-189`

### D. Resident host, Task 7: partial-display edits and host controls
The `apply_observed_commands` adapter and observed-command adapters for product operations using
Task 1 digests, exposing only supported partial commands (closure, global, import, delete, withdraw,
history, field and coupling operations return `NeedsCompleteOperation` or take the explicit Full
path); fallible UUID and JSON parsing at the boundary; graph controls (Full or Neighborhood,
projection, hops, Manual or Host-driven, intervals, Save, Refresh) with coverage wording and Inspect
and Demand; canvas and scene sourcing `presentation_graph()`; endpoint controls on the admitted
Djinn path. Demand does not prove an edit's dependency completeness. The lane also added IndexedDB
lifetime admission and a version-2 cutover; the port map defers browser composition.
- Done when: the shell exposes residency, edits, manual and host refresh and clear recovery actions,
  exercised through endpoint, client and local browser rather than inferred from library tests;
  edits work from the partial display, reopen and undo with the same truth and history as
  complete-session edits, refuse invalid or stale targets, and recover from failed publication
  without duplicate edits; the Full peak and retained-writer costs are stated plainly.
- Needs: C
- Rulings: resident 1, 2, 4
- From: `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:43,48,61,73,674-831`;
  `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:145-174`

### E. Resident host, Task 8 qualification, then Turnstone's authority migration
Task 8: a browser IndexedDB test (Firefox only, no Node or Chromium substitute); DirectoryBackend
and RedbBackend prepare, write, reopen and undo including a recovered poisoned batch, other backends
and unproven ownership doing no combined writes; a real Djinn route with a retained client (two
panes, scoped roots, Full, demand, stale input, context-only notices); headed Firefox shell runs on
fresh test IndexedDB; a 64-node fixture at Full and Neighborhood(0/1) with process RSS reported
apart from reader savings; a fresh reviewer across the five review-focus classes (hidden incident
change, later command targeting an earlier creation, write that commits then errors, empty or
missing authored roster, coverage update during refresh). Then, after the first (Graphshell and
Djinn) host, the same authority boundary is adopted in Turnstone's custom Canvas lifecycle, keeping
legacy save, migration and refusal behaviour and never letting two persistence authorities write one
graph; authored scoped projection roots come first, since Turnstone's identity Forme roots every
Surface and cannot show neighbourhood savings.
- Done when: source-qualified implementation, real host and client, and actual Firefox evidence
  support each claimed outcome, and all deferred scope stays explicit. Turnstone: edit, store,
  publish, notice, refresh and close or reopen preserve identities, authors and controls through a
  real endpoint and a real Turnstone profile; failed publication or refresh and stale completion
  preserve the previous view.
- Needs: C, D; Mark's approval of the exact candidate before any push or merge.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:87,90,94-102,833-893`;
  `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:49-52,57-60,90-113,145-149`

### F. Scoped transactions, safe generation collection and the deferred bounded writer
A default-refusing scoped transaction-read seam over existing backends (declared exact values and
key-only prefixes, explicit budgets, overflow refused, never truncated); in-flight generation loads
protected transactionally; reachability covering node records, navigation entries, visits, owners
and retained pending owners; references rechecked in the deletion transaction; only residency
materialisations deleted, never baselines, journals, history, archives or payload blobs. Collection
stays disabled until protected-reader adoption at a safe migration boundary; nonexpiring crash
references may hold storage until exclusive-owner recovery is proven (a timeout proves nothing). A
writer that never materialises the whole graph is a distinct, larger contract, deferred and
unscheduled; neither a partial UI nor a partial reader is advertised as one.
- Done when: publish, load and collect races preserve referenced records; shared immutable records
  and navigation and pending closure survive; corruption and unsupported capabilities delete
  nothing; failed or restarted cleanup is idempotent; session history reopens exactly; redb,
  Directory and Firefox IndexedDB (sealed wrapper and cancellation included) are exercised. Bounded
  writer: operation dependency declarations are complete, journal and materialisation update
  incrementally, history reconstruction is bounded, and unsupported or global operations refuse.
- Needs: C-E, for protected-reader adoption; Mark's memory choice (below) for the bounded writer.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:53-56,69-71,145-174,191-218`;
  `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:87-88,907`

### G. Query views and additional strata
First target: the read-only tagging and query view over the existing strata, reusing the saved
SELECT membership adapter, coverage notes and source revision, with an explicit
resource-to-appearance policy; its inspector separates a lifted resource link, a surface link and a
surface-resource binding. Definitions are shared Mere Resources edited in Graphshell Workbench;
links and cross-stratum edges carry distinct configurable stroke cues independent of theme colours
(Mark, 2026-10-13). Strata draft: candidate strata (assertions, occurrences or events, composition)
each need four answers before the model extends: identity, owned facts and rules, joins to other
strata, and who edits it and how persistence, replay, sharing and residency keep it; a stratum
registry would need identifiers, decoder behaviour and migration rules.
- Done when: the first target passes native model, owner persistence and headed browser receipts
  covering two surfaces showing one resource, an undrawn resource, distinct owners' equal tag
  labels, parallel attributed assertions, navigation to a different resource, exact empty membership
  and incomplete coverage, with read-only creation preserving surface identities and Forme
  placement. Strata draft: a candidate stratum has all four answers.
- Needs: first target, final supplier qualification and publication; strata draft, the link model
  plan's answers on assertions and pair-level occurrences.
- Rulings: 3, 8, 15, 20, 46
- From: `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:4278-4290,4320-4344,4383-4456`

### H. Navigation lineage on a live host
The kernel half exists (`SharedNavigationMemory` on `Graph.nav`); Phases 1-4 wired the retired
meerkat and orrery path and have no host. P1, navigate the focused node in place. P2, gestures
(context menu, middle-click, Ctrl-click, Ctrl or Cmd-Enter) mint a node with a navigated-from edge
anchored at the origin's current visit, stacked into the active slot, with a no-origin path. P3,
relation tiers drawn distinctly (Semantic strongest, then Containment, Arrangement and Provenance,
Traversal weakest, Imported its own style; re-check the donor edge-style code and the orrery's
per-pair collapse). P4, previous and next as most-recently-used order over Traversal (long-term
history is Eidetic's), controls in gloss, a lineage swatch. P5, diffable visit-tree similarity,
unscheduled.
- Done when: P1, navigating the focused tile changes its page in place, builds the node's history,
  back and forward walk it with forward-fork preserved and no stray node minted. P2, gestures mint
  distinct nodes with correct anchors, stacked into the active slot, a revisited origin getting a
  fresh anchor. P3, edges render by tier, visibly distinct. P4, previous and next walk the MRU from
  gloss, distinct from within-node history, and a node's lineage renders as a swatch.
- Needs: a live host (question below); the curation plan's gloss host for P4.
- Rulings: 2; the 2026-06-05 model: a node is a browsing surface with UUID identity, going back then
  navigating forks the visit tree and never truncates it, navigating in place mints no node, and
  `node.url()` and `current_history_url()` stay single-source.
- From: `archive_docs/2026-10-28/2026-06-05_node_navigation_lineage_wiring_plan.md:10-19,38-50,59-112,106-131,142`

### I. Substrate ladder: G3, G4, G5
G3 is met in substance (Woodshed's practice history is a stemma) pending a ruling on its reworded
clause. G4 has only its export half in `chartulary::rdf`; the round-trip gate has not been re-passed
over the generic substrate. G5 is the long tail: mere implements the traits on its web node,
re-derives history and capture over the spine, retires the in-tree node-lineage and
`graph/history.rs` and `capture.rs`, and the analytics layers retarget. The fork primitive and
provenance header are Eidetic roadmap items; Tier 2 layers follow the core.
- Done when: G3, user-authored content lives in a substrate graph durably; G4, the losslessness gate
  passes over the generic substrate and a woodshed note citing a mere document exports as real
  JSON-LD; G5 as written (meerkat on the substrate graph, no behaviour change) cannot be met.
- Needs: the G3 and G5 restatement ruling.
- Rulings: substrate decisions 1-4 (a fully generic core; mere re-bases last; one history spine; the
  names chartulary, stemma, scholia; G0-G4 never block on G5)
- From: `archive_docs/2026-10-28/2026-07-08_generic_graph_substrate_plan.md:12-32,42-44,122-131,144-172,189-251`

### J. Ownership repair, lane L1 remainder
Beyond the landed kernel and chartulary slice, L1's second slice (graphshell host admission Q056 and
Q057, and the five authority entries the first round named) is at its owner question round of
2026-10-28, each fork with a proposed owner and no answer from Mark yet. A001: one `pub(crate)`
primitive per navigation operation under the live and replay arms. A003: the facet holds recency and
Stemma the trail only, and the session clock, which has no writer, is given one or removed. A012:
the kernel's checked load as the one structural validator, facets installed through one checked
primitive, every archive save running it. A016: one canonical snapshot form at the source. A017: a
typed id remap on `CapturedDelta`, exhaustive beside `Touched`. Also the pictograph and
graphshell-web consumers, Q014 (re-document only), and two public-API removals for the next
mere-kernel release (`store`, `replay_delta`). L1 answers each with a test: two windows overwriting
without detecting the conflict (Q253, Q214, Q061, A004, A011, A028); a projection with incomplete
data saving itself as the complete graph (Q012, Q017, Q257, A067); a failed write advancing
in-memory history (Q240, A550, A551, Q047, Q234, Q380, A1119); a second host editing a session by
presentation alone (every lane).
- Done when: every finding in the slice is repaired with a test or re-documented with its reason;
  duplicated implementations in its authority entries are reduced to one owner or recorded as
  deliberately separate, the owner named in code documentation; touched crates pass in the worktree
  with pins holding; a dated successor receipt updates register status.
- Needs: Mark's answers to the owner round before code.
- Rulings: the L1 rulings under Constraints
- From: `2026-10-20_ownership_repair_plan.md:55-80,106-182,198-209,236-243,407-514,603-609`

## Open questions

- Is the first query-view slice qualified? The status says final supplier qualification and
  publication are in progress and later card, reader-wheel and supplier repairs need a fresh bundle;
  the body says it is qualified (Genet b6, Netrender 6a). From:
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:4280-4290,4449-4456`
- Which living host takes lineage P1-P4, meerkat and orrery being gone? Does Q001 (landed
  2026-10-26) close the caveat that removed-node owners are kept and a `delete_owner` bug makes
  `to_snapshot` panic? Is per-operation rehydrate acceptable? From:
  `archive_docs/2026-10-28/2026-06-05_node_navigation_lineage_wiring_plan.md:10-19,61,126,142`;
  `2026-10-20_ownership_repair_plan.md:216,592-602`
- How are G3 (reword "through muniment" to durability) and G5 (the meerkat clause) settled: restate
  in place, or move G5 into a mere re-base plan? Five design points were never stated as settled:
  family-registry mechanics, edge multiplicity, rkyv, crate granularity, whether the spine feeds
  stemma. From:
  `archive_docs/2026-10-28/2026-07-08_generic_graph_substrate_plan.md:94-100,207-219,248-251,276-284`
- Is the archived petgraph-RDF Phase 3 done on the shipped spareval path, or reopened? Is the kernel
  `apply.rs` split (1938 lines) still wanted, and is `merge_snapshots` promoted to
  `Graph::merge_from`? From: `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:1517-1525`;
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:155-157,1050-1054,1060-1061`
- Resident fork F3: where is a selected projection validated before Neighborhood is enabled, the
  host being unable to resolve projection names? When is Ruling 4's double full-graph encode fixed
  (candidate: read digests from the last published catalog when its generation is current)? Mark was
  asked for the first-milestone memory choice (temporary Full recommended); no answer is recorded.
  The plan's status lines also disagree (the 2026-10-15 line says only Task 1 and parts of Task 4
  are ported, the port log Tasks 1-3 and the host half of Task 4); this plan follows the port log.
  From: `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:14-18,61,63-75`;
  `archive_docs/2026-10-28/2026-10-09_graph_semantics_followons_plan.md:164-169`
- Do the lane's IndexedDB lifetime admission and version-2 cutover carry to main, and where does the
  browser pump live? Does main need the lane's default-off coverage feature on Knot? From:
  `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:43,48,831,948-950,970`
- Which of the archive-tag lane's unresolved findings apply to the port: the
  dirty-settings-held-reader case, failed-store and hidden-save recovery, Forme scroll routing, the
  headed intake exact-empty check, Firefox graphics memory pressure, layout under 480 px, the
  untested `preventScroll` repair on `a11y-mirror-prevent-scroll`? From:
  `archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md:1008-1060`
- What mechanism aggregates tag vocabularies across meres (ruling 20 points at the moot direction),
  and where does the pending-link retention setting surface? From:
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:2520-2530,2672-2677,2880-2891`
- Three L1 forks are either/or: the session clock (give `last_session_visited` a writer by
  installing the launch counter, or remove the clock, `PersonaSettings.session_count` and
  `KeepSessions`, touching the captured navigation format); Q056 (the host's vocabulary becomes part
  of what admission answers, `ActionNotOffered` written before anything closes, or an Accept
  followed by a close is read as not served); Q057 (prose and test claim only what admission
  enforces, or the responder supplies freshness on every carrier, a Notochord change). Q056 and Q057
  also sit in `2026-10-28_source_audit_repairs_plan.md`'s routing to the Graphshell host plan: which
  plan takes them? From:
  `2026-10-20_ownership_repair_plan.md:440-448,493-514`
- The composition candidate stratum has no answer to the four questions, and a general registry of
  cross-stratum relation records would need its own contract. The frozen link still refuses an empty
  handle that checked load accepts; not separately ruled. From:
  `archive_docs/2026-10-28/2026-10-04_graph_semantics_plan.md:4322-4344,4383-4412`;
  `2026-10-20_ownership_repair_plan.md:280-281`

## Findings

- 2026-10-09: Graphshell's `MereHost` owns GraphSession and Djinn shares it; Turnstone's Canvas has
  its own persistence. A partial reader beside a complete writer keeps the whole graph in the
  process, and editing still peaks at Full. Muniment's IndexedDB transaction snapshots every
  object-store value, so scoped reads must precede any bounded-memory claim (followons `:49-72`).
- 2026-10-15: `GraphSession::transact_change`
  (`crates/system/pandect/src/graph_session_transaction.rs`) stages, checks and installs atomically
  on every backend; a backend that commits then errors leaves the host at its acknowledged state and
  a retry is refused, not duplicated (resident `:20-48,63-75`).

- 2026-10-30: bounded live resource-pair capture supports Turnstone's exact
  assertion recovery without copying unrelated graph payloads. Unlike a durable
  snapshot, replacing a live pair must preserve session-only arrangements too.
  The public kernel API checks edge count and serialized bytes before cloning
  the selected payloads. These are operation bounds, not a total process-memory
  ceiling. The consumer's authority, storage and recovery contract stays in
  `eidetic/design_docs/eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md`,
  P4a M4; supplier and consumer evidence has one home in
  `eidetic/design_docs/archive_docs/2026-10-30/2026-10-11_eidetic_development_plan_receipts.md`.

## Progress

- 2026-10-28: cut from the six sources named in Status; nothing new landed.
- 2026-10-30: published bounded live resource-pair capture and preservation tests
  for the model-free structured-record recovery consumer.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- Production journal persistence (the G5 follow-on), deferrable only until a real log exists. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:461-464`
- The E2 phase 2 relation delta journal, gated. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:826`
