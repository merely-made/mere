# Engine split plan: Mere as the semantic-web engine, Graphshell as the host, Inker and Conatus as suppliers

**Status:** 2026-11-07: plan. Nothing has moved. D1 to D10 are ruled; the engine-to-workspace
contract is deferred by D4; the Pandect sidecar owners are open.
**Status:** 2026-11-07: plan. Nothing has moved. D1 to D25 are ruled (D21 reverts D20); `merely-made/inker` holds the twelve
crates with history (main `3109942`, pushed); the plan is on origin's main
(pushed 2026-11-07 at `9e790a152`); Eidetic main carries D12 (`71188bb`) and Knot main the D15
repin (`2c703c4`); P1 and P2 lanes have proved their crates standalone in worktrees (nothing merged);
`merely-made/seiche` holds the seven spatial crates (main `a6b8641`) and Mere takes `seiche` and
`numen` from it (P2's cut, below); `merely-made/inker` holds the twelve Inker crates (main `436c841`) and Mere
takes them from it (P1's cut, below); the engine-to-workspace contract
is deferred by D4; the Pandect sidecar owners and the lanes' forks are open.

**For:** making Mere what its name and design record describe, the engine for a person's
linked-data graph, in the posture Genet takes toward the web platform: dereference, parse,
model, interpret, present through a host, serve nothing. Everything about how a person holds,
arranges and moves through that graph leaves for the host. Mark, 2026-11-07: "me personally,
i think splitting is ok. pandect was becoming another grab bag like djinn anyway. we should
keep an eye out for crates that have drifted from a clear intention, too.
redefining/refactoring them is possible, but i agree that planning now beats not having a
plan later." The end state was converged with two independent layouts the same day; where
they disagreed, the forks went to Mark (Decisions).

**Not here:** the vocabulary-pack model (the [linked-data platform lessons brief](../research/2026-11-06_linked_data_platform_lessons_brief.md),
D1 to D32, and the declared-lens plan it owes); the family dependency cycles
([cycle breaks plan](2026-11-06_cycle_breaks_plan.md), whose D8 and D9 this plan refines);
the resident's own split ([resident split plan](2026-11-05_resident_split_plan.md)); domain
adoption and the domain contract ([adoption plan](2026-11-05_domain_adoption_plan.md),
[contract](../technical_architecture/2026-11-06_domain_contract.md)); Djinn's slimming
(Armillary); the stance rewrite, which becomes the engine's charter and is written when the
engine crate exists.

Receipts go to `archive_docs/receipts/2026-11-07_engine_split_plan_receipts.md` (DOC_POLICY §8).

## Constraints

- **Cycle-breaks rules bind every move.** Family rows use the canonical URL with
  `branch = "main"`; cut first, not lockstep; moved code keeps its tests; no manifest churn
  outside a phase; docs move with the owner of the subject; dev-only edges count as cycles
  (cycle-breaks D6).
- **Layering, bottom up:** Eidetic (Stickleback, Muniment, the domain interface, the pack
  header), then Mere (engine, reservoir service, Pandect's graph-domain lifecycle), then
  Conatus and Inker (suppliers beside Mere; Inker depends on Genet and may depend on Mere's
  dataset surface, never on the host), then Graphshell (host contract and hosting), then the
  applications. Nothing lower names anything higher. The source homes map's layout test
  applies: Turnstone and Knot resolve one source per shared package, and the count of
  repositories repinned for a change to reach Turnstone is recorded per phase.
- **Repository count is a cost** (D2). This plan adds Inker and Seiche and reuses the revived
  Graphshell; D7 pays for them by folding smolweb, emblem and wavicle into their consumers,
  so the family nets one fewer repository (24 from 25).
- **One intention per crate, and one crate per intention** (D1, D10). Each crate touched
  states what still fits its original job; the rest gets an explicit disposition: merge into
  the neighbour whose job it serves, move, split at an adapter, or retire. Merging is the
  default where a crate's job is one module of a neighbour's and the merge frees a name. A
  crate that has started making decisions that belong to another owner is split, not
  relocated. A distinct purpose that needs a name takes a freed family name before a new
  word is spent.
- **Ownership and cutover are planned together.** Each split records who accepts the write,
  what survives reopen, who consumes the contract, and which package or wire identities carry
  existing users across. Moved packages keep their names during cutover; a workspace crate
  that leaves Mere may drop the `mere-` prefix (source homes map, identity preservation). The
  new `mere` package surface is the one deliberate break.
- **Measure before the join moves.** The 2026-06-18 benchmark's lesson stands: hold the
  dataset once per revision, hand the host an indexed projection, and measure load, mutation,
  projection refresh and frame cost separately. UUIDv5 is a durable binding, never a
  per-frame join. An interned arena is a candidate if the baseline misses budget, not a
  commitment.
- **Standards follow the suites.** Engine capabilities ship behind a conformance ledger row:
  spec revision or pack version, supported profile, fixture suite, result. No capability is
  claimed without its row.

## The end state

| Home | Holds | Intention |
|---|---|---|
| `merely-made/mere` | the `mere` engine crate; the reservoir service; `pandect` as the graph-domain revision lifecycle | dereference, parse, model, interpret; the person's profile of it |
| `merely-made/graphshell` | the host layer as a library (D23): the contract root below (cycle-breaks D9's leaves plus tinct) and the hosting above (workspace kernel, Cambium family, projections, chrome, resident assembly); no application | how a person holds, arranges and moves through graphs |
| `merely-made/turnstone` | the reference host duties (D23): the Graphshell reference application, its loopback acceptance view and the MCP server, beside Pelt | the product that exercises the host layer end to end |
| `merely-made/inker` | engine selection, readers, document canvas, smolweb, lexing, clipping | what a page says and how it reads |
| `merely-made/seiche` (D6) | bodies, voxels, fields, force integration, lit tenant | the reusable spatial runtime |
| unchanged | Genet (web engine), Eidetic (store substrate, domain interface, pack header), Armillary (Djinn), Dramatis (custody), Distillery (works), Gemot, Murm | as today |

### The engine's stages

Mapped against Genet's pipeline. Each stage is a module of the `mere` crate or a crate beside
it; each ships behind ledger rows.

| Stage | Genet | Mere | From today |
|---|---|---|---|
| Acquire and preserve | fetch | resolve an IRI behind a narrow loader interface; negotiate and parse a representation; retain source and dataset with retrieval provenance | `linked-data` ingest; loader interface new; I/O stays with the host (`mere-fetch`) and reader (`mere-pageload`) |
| Model | DOM | RDF 1.2 dataset: named graphs, reifiers, revisioned reads; assertion handles, scope, evidence, withdrawal, the asserter-only rules of the link model | `graph-kernel` resource half, `edge_data.rs`, journal, validators |
| Query | selectors | SPARQL 1.2 query and update over the dataset; typed results | `linked-data/query` and its `QueryableDataset` adapter |
| Formats | parsers | Turtle family, RDF/XML, JSON-LD 1.1 with contexts; serialization | `linked-data` codecs, `oxttl`, `oxrdfxml`, `oxjsonld`, `json-ld` |
| Load a semantic profile | stylesheets | a versioned user-agent pack plus admitted vocabulary, shape and rule packs; record what was selected | `predicate_registry.rs`, `predicate_declarations.rs`, `vocab.rs`; pack loader new (lessons D6 to D16, D31, D32) |
| Interpret and diagnose | style resolution, validator | RDFS, OWL 2 RL and pack rules into a separate derived layer; SHACL against a declared view, returned as diagnostics that domain policy may act on | lessons D24 to D29; new |
| Resolve identity and verify proofs | origins, credentials | WebID and supported DID methods; proof verification and canonicalization where a profile calls for it; custody owns keys and grants | lessons D17 to D19, D23; Dramatis; new |
| Script surface | WebIDL | a dataset WIT interface over query and mutation | new; `mere:script`, wire and codegen stay with the host (D2) |
| Profile | history, bookmarks | the reservoir service; Pandect's graph-domain lifecycle | `pandect` split (P4) |

Not in the engine: layout, paint, physics, the surface graph, sessions and view state,
fetch I/O, cookies. A browser engine renders through a host and serves nothing; so does this.

### Crate dispositions

Every non-probe crate in Mere today (62), by receiving home. "Split" names the seam.

**Stay in Mere, the engine.** `graph-kernel` (resource half only; the surface half leaves,
see Graphshell), `linked-data` (folds into the engine crate), the dataset WIT interface (new),
the pack loader (new).

**Stay in Mere, beside the engine.** `pandect` (graph-domain revision lifecycle only: admission,
fold, branch, merge, replay; its reservoir half becomes the reservoir service crate, its
sessions, manifests, view sidecars, worker declarations, settings and furnace paths are traced
to their owners in P4), the reservoir service (new crate from `pandect::reservoir`:
persona and domain catalog, access, store lifetimes, sealing, replication), `mere-framing`,
`state-witness`, `mere-facts` (the persona's facts domain; its admission policy stays with
the domain owner, as the contract says). The reservoir service crate is `catchment` (D9).

**Graphshell: the host.** Workspace half of `graph-kernel` as the workspace kernel, which
grows out of `incipit` and takes its name (D8: surfaces, trails, arrangements, bindings to
resource references by UUIDv5; the graph and session ids stay reachable behind a minimal
default feature so Cleromancy and Fleece keep depending on ids alone),
the `mere` facade's navigation, trail and roster logic (its visual and workbench APIs to the
view and composition crates; no new facade), `mere-cartography`, `pictograph`, `mere-forme`,
`mere-curation` (merged into `incipit`, P0 done), `platen`, `uxtree`, the Cambium family (`meristem`, `cambium`,
`cambium-rootstock`, `cambium-winit`, `cambium-a11y`, `cambium-desktop`, `cambium-web`,
`workbench`, `hest`, `backstep` (kept and grown under D27 into the editor's ephemeral state: snapshot undo, the
host clipboard contract, chrome's linear back/forward; P0 lane `p0/backstep`, not merged), `mandrake`, `sprigging`, `tinct`, `persona-picker` (merged into `cambium`, P0 done),
`mere-view`, `mere-overmap`), `tabard`, `mere-chrome`, `mere-registry` (examined feature by
feature in P0; a registry may carry several owners' contracts but makes none of their
decisions; P0 done: trimmed to `diagnostics` and `observations` under its own name, the five
dead registries deleted, the mod loader moved into `document-host`), `mere-surface-api`, `settings-wit`, `ux-events`, `mere-fetch` (the host's fetch
service, keeping cookies and the engine-flip state; RDF resolution policy is the engine's),
`graphshell-host`, `djinn-testkit`, `mere:script` with `script-wire` and `script-witgen`,
`app-host`, `document-host`, `scenograph`, `scenomise`, and the ports `mere-resident` (the
concrete assembly), `pelt`, `tabard`, `dramatis`, `gazette-ledger`, `moot`, `distillery`.
Under D23 the reference host application (`ports/graphshell`) and `graphshell-mcp` go to
Turnstone instead, so the Graphshell repository holds no application. The contract leaves cycle-breaks D9 already assigns
(`chirograph`, `graphshell-client`, `graphshell-endpoint`'s default surface, `sceno`,
`scenotime`, the WIT world) are the same repository's bottom layer.

**Inker.** `inker`, `document-canvas`, `graft-engine`, `scrying-engine`, `weld-engine`,
`nematic`, `illume`, `knot-editor-host` (after its Cambium editor integration is split off to
the host), `mere-lanes` (taking reader styles and owning its reading-anchor type; the host
adapts Tabard at the edge), `mere-pageload`, `mere-fleece` (generic traversal, clipping and
capture; its `crawl-graph` intake adapter goes to the host), `script-rhai`. Inker emits a
document, a predicate-bearing link or a JSON-LD block with its base address; the engine
consumes RDF; the host decides what to keep.

**Seiche** (the spatial repository, D6). `conatus`, `modulus`, `nisus`, `tenant`,
`quint-shaders`, `numen`, `seiche`. Isocosm's Eponym and Mesocosm already consume the family
`numen`, `seiche`. Isocosm's Eponym and Mesocosm already consume the family
without the graph.

**Elsewhere.** `mere-alembic` to Distillery (it says it is Distillery's component);
`mere-comms` per cycle-breaks D4 when unpaused; `chirograph` per D9. Empty directories
`crates/crawl` and `crates/import` are removed. Forks in `Code/crates`: `boa`, `bole`,
`vano` follow Genet's scripting adapters; `xilem` follows Cambium; `nexus`, `kiss3d`,
`vello` follow Conatus and the canvas; none enter the engine. New under `Code/crates`: the
W3C `rdf-tests`, SPARQL, `json-ld-api`, `data-shapes` and RDFC-1.0 suites, vendored the way
`wpt` is, as the engine's test fixtures.

## Phases

Order follows D4: Inker and Seiche may move at once, the hosting may move, and the
engine-to-workspace contract is settled after. P0 runs first because every later phase
cites it. The consolidations (C) ride the phase that creates or reorients their receiver.

### P0. Drift audit and the conformance ledger

For each crate above: one line stating what still fits its original job, and a disposition
for the rest, merging by default (D10). `mere-registry` feature by feature; `mere-fetch`,
Djinn's assembly and `pandect` as the named seams. Candidate squishes from today's
measurement, to confirm or refuse line by line: `mere-forme`'s arrangement authority and
`mere-curation` into `incipit` (workspace truth; frees two names); `mere-cartography`'s
contracts into `pictograph`, their only implementer (frees one); `backstep` and
`persona-picker` into the Cambium crate they serve (frees two); `mere-registry`'s eight
registries to their owners (frees one); the `mere` facade dissolved as planned. Each line
records the name it frees and whether a distinct purpose in this plan can take it. Open the conformance ledger as a table under
`mere_docs/testing/`, with the columns from Constraints, seeded with the RDF 1.2, SPARQL 1.1
and JSON-LD 1.1 suites against today's `linked-data` adapter, so the first rows record the
starting point.
**Audit of 2026-11-07** (58 crates plus 8 the plan had not placed; the table is in the receipt
file, spot-checked). Confirmed as written: `mere-curation` into `incipit`; `persona-picker`
into `cambium`; `mere-registry` freed by retiring its five never-enabled registries (`input`,
`knowledge`, `layout`, `lens`, `protocol`, 3,876 lines with no consumer in six repositories)
and moving the three live ones (`mod-loader` to `document-host`, `observations` and
`diagnostics` to the resident). Corrected: `backstep` cannot merge into `cambium` (cycle:
`scenograph` and `scenomise` depend on it and `cambium` on them); `meristem` is the candidate.
Reopened as forks: `mere-forme` (into `incipit` or `platen`) and `mere-cartography`, which
holds six layout strategies, not only contracts, while `pictograph` is 93% graph canvas
(63.8k of 68.2k lines) and 7% node faces. Not placed before, now to place: `mere-edit`,
`graphshell-door`, `block-host`, `graphshell-blocks`, `graphshell-mere`, `tabard-workshop`,
`tabard-desktop`, `graphshell-client-tests`. Further squishes the audit ranks, each a fork:
`mere-framing` into `state-witness`; `gazette-ledger` into the `graphshell` port;
`mere-alembic` with `mere-athanor`; `document-host` and `block-host` into `app-host`;
`settings-wit` into `mere-surface-api`; `tinct` into `tabard`; `uxtree` into `platen`;
`mere-overmap` into `mere-view`; `djinn-testkit` to Armillary; `mere-moot` to Gemot;
`ux-events` retired (no dependents). The Inker crate lines come from the P1 lane.
- Done when: every crate in Crate dispositions has its line; the ledger exists with its first
  rows and their results; the suites are vendored under `Code/crates` with their revisions
  recorded.
- Needs: nothing.

**Inker crates** (P1 lane, 2026-11-07; lines of Rust under `src`, consumers across Mere,
Turnstone and Knot manifests). Proposals for Mark to confirm or refuse; F1 to F4 are P1's forks.

- `inker` (9.8k; 9 Mere crates, Turnstone, Knot): fits, selecting engines and carrying the
  document model, surface-engine contract, cue, sniff, flip and the pure `statements` walk the
  engine ingests. Stays whole; frees nothing.
- `document-canvas` (6.5k; lanes, platen, tabard-workshop): fits, laying out
  `inker::EngineDocument`. Stays: folding it into `inker` would put parley and netrender under
  every engine-controller consumer (fetch, platen, uxtree, linked-data). Frees nothing.
- `graft-engine` (1.0k), `scrying-engine` (1.5k), `weld-engine` (1.9k) (Turnstone optional;
  pelt-desktop takes scrying): each is one `inker::SurfaceEngine` adapter over one foreign
  producer, a module's worth. Merge into `inker` as feature-gated `surface` modules, keeping
  `inker`'s default graph wgpu-free; frees three names, none claimed on crates.io (all
  `publish = false`). No distinct purpose in this plan needs them.
- `nematic` (10.5k; lanes, uxtree, document-canvas dev, Turnstone, Knot): fits, lowering
  smolweb formats to inker documents over `errand` (its only direct smolweb crate). Stays;
  frees nothing.
- `illume` (2.0k; cambium optional, graphshell, Knot): fits, the portable lexer. Took
  `knot-editor-host`'s Markdown reuse-lexer and `full_pack` with their two tests, behind the
  new `markdown` feature (D17).
- `knot-editor-host` (0.3k; graphshell port, Turnstone, Knot's `knot-document`): retiring
  (D17). The lexers are in Illume and re-exported here; `KnotEditor` and `KnotReadout` go to
  Knot's `knot-document`; when the consumers have moved, the crate retires and frees
  `knot-editor-host` (published 0.1.1).
- `mere-lanes` (8.4k; graphshell, pelt, tabard-workshop, Turnstone): fits, the reader and
  smolweb lanes over the document canvas. Owns its `ReadingPreferences`, `ReadingAnchor` and
  reader styles (D16) and the smolweb palette over Graphshell's `tinct` (D19); no Tabard
  dependency remains. Its `remote` fetcher
  (netfetcher and errand, 0.4k) overlaps `mere-fetch`'s transport and is traced in P3. Stays;
  frees nothing.
- `mere-pageload` (0.8k; mere-fetch re-exports it, pelt-core, Turnstone, now Fleece's crawl):
  fits, the sans-IO page load and fetch vocabulary. Stays: folding it into `inker` would make
  `mere-fetch`'s optional `inker` edge mandatory. Frees nothing.
- `mere-fleece` (7.0k; lanes, moot, pelt, graphshell, the facade, Turnstone): fits, bounded
  traversal, retained traversal, clipping, capture and the Eidetic bridge. P1 cut its
  `crawl-graph` intake (below). `structured-data` (0.6k, no consumer in any manifest) projects
  JSON-LD blocks into graph contributions through `mere-rdf`, which is the engine's job by this
  plan's own sentence (F3). Stays; frees nothing.
- `script-rhai` (1.2k; graphshell, Knot): fits, the Rhai `BlockEvaluator` and `ValueEngine`.
  Its `command` module collects `hest::Invocation`s from a host-offered `CommandSet` over
  `hest` from Graphshell's contract root, which Inker may name (D19 settles F4). Stays whole;
  frees nothing.

**Spatial crates (P2 lane, 2026-11-07).** Lines are `src` lines; dependents are manifests in
Mere, Isocosm, Cleromancy, Knot, Murm and mer3ly.
- `conatus` (5,959): fits. Host-neutral 3D bodies, collision, queries and fixed step over a
  private Rapier 3D (`body`, `clock`, `command`, `engine`, `schedule`, `world`); Eponym,
  Mesocosm and Isometer consume it. The `resident` module (CubeCL n-body lane) has one
  consumer, seiche's `gpu` feature; it is a module, so it stays and frees nothing.
- `nisus` (635, serde only): fits, and its job is one module of conatus's (voxel chunk and
  edit values; conatus takes it under `bodies`). Merge candidate into conatus behind a
  voxel feature with `bodies` off, freeing `nisus`. Mesocosm takes it today precisely to stay
  free of a physics backend, so whether a conatus feature honours that is a fork (P2's).
- `modulus` (2,239, bytemuck only): fits whole. Sparse-brick ABI and its WGSL traversal;
  Eponym and Isometer consume it, nothing in Mere does. Tenant is the nearest neighbour,
  but merging would put tenant's kiss3d fork under Isometer's lens. Stays; frees nothing.
- `numen` (4,042): fits. Field truth and evaluation; consumed by seiche, `pictograph` and
  `graph-kernel` (its field store). Folding into seiche would put Rapier under the kernel.
  Stays; frees nothing. Its edge from `graph-kernel` becomes Mere naming Seiche (P2's forks).
- `seiche` (28,781): fits. Kernel-free holds against its manifest: normal dependencies are
  rapier2d, petgraph (for `NodeIndex` only), numen, euclid, tracing, and optional armillary,
  conatus, wgpu, Burn and serde; no kernel. Its one Mere edge is the dev-dependency on
  `state-witness`, for the `binding_replay` examples. Its `lib.rs` doc still describes the
  tick mirroring into "the graph" and petgraph "inside kernel"; that wording is fixed with
  the move. Stays whole.
- `tenant` (1,275): fits. Lit body renderer over the mark-ik kiss3d fork (pinned by rev);
  Eponym's client and Mesocosm's genet host consume it. Rendering rather than physics, but
  inside D5's set. Stays; frees nothing.
- `quint-shaders`: no longer exists. Folded into `conatus::resident` and deleted by
  `eae87153` (2026-08-31); nothing moves.
- Names a merge would free: `nisus` only, if the fork goes that way.

### P1. Inker

Cut Fleece's `crawl-graph` intake to a host-side adapter; move reader styles and the
reading-anchor type into `mere-lanes`; split `knot-editor-host`'s Cambium integration. Then
move the Inker crates to `merely-made/inker` with their tests, and repin Mere, Turnstone and
Knot to the canonical source. Mere receives Inker as a supplier.
- Done when: Inker builds and tests with no path or git dependency on Mere's host or
  workspace crates; Turnstone and Knot resolve Inker from one source; the reader and smolweb
  headed receipts that exist today pass from the new home; the moved packages keep their
  names.
- Needs: P0's lines for the Inker crates.
- State (2026-11-07): **Mere's half is cut.** Mere deleted its twelve copies and the three
  design areas and takes the crates from `inker.git` (`branch = "main"`, locked at
  `3109942`; `436c841` above it adds only Inker's DOC_POLICY and DOC_README). Every moved
  file matched Inker's copy except `document-canvas`'s text test, which reads Inker's own
  Roboto (F5, settled there). Knot's pinned revision still names five of them through
  `mere.git`, so Mere's `mere.git` patch table redirects those five to `inker.git`;
  Inker names the Cambium family through `mere.git`, so the Graphshell web workspace now
  restates eight Cambium-family path redirects. Turnstone and Knot repin is open.

**Lane state, 2026-11-07 (worktree branch `worktree-agent-a61f20e98db5486a2`; receipts in the
plan's receipt file).** Landed in the worktree: Fleece's `crawl` takes `Fetched` from
`mere-pageload` (the type `mere-fetch` re-exports) instead of depending on `mere-fetch`; the
graph binding became the generic `crawl::session::CrawlSession<D, T>`, retiring the
`crawl-graph` feature and its `incipit` and `mere-rdf` edges (no consumer enabled it); the
single-hop lowering moved with its test to `mere-fetch`'s new `fleece-crawl` feature; the
Scenograph rule-runner test moved from `script-rhai` to `scenograph` as a consumer test, so
the dev edge back to the host is gone. Under D16 `mere-lanes` owns `ReadingPreferences`,
`ReadingAnchor` and reader styles (`ReaderRole`, `ReaderFontStyle`, `ReaderStyles`, with
family lists the host resolves); `tabard-workshop` converts Tabard's typography at its edge.
Under D17 the Markdown reuse-lexer and `full_pack` moved with their tests to Illume's new
`markdown` feature; `knot-editor-host` re-exports them and is marked for retirement. A
standalone workspace of the twelve crates (copied to the system temp directory, Genet and
smolweb, `merely-made/errand`, from their canonical URLs) checks and tests green (769 tests).
On main after D15 (`4b99ce9f2`) Mere checks green; the Graphshell port converts at its five
D16 boundary sites in `retained_content.rs` (Mark's option (a)), its persisted anchors staying
Tabard's. Two Inker edges into Mere remain, plus one file edge; `mere-lanes -> tinct`
and `script-rhai -> hest` reach Graphshell's contract root, which D19 allows:

- F6, settled by D19 (Mark's smallest option): the smolweb palette (`SmolwebTheme`,
  `SmolwebPalette`, `ModeProfile`) moved into `mere-lanes` (`src/smolweb/palette.rs`, with its
  four tests) over `tinct` from graphshell.git; the lanes no longer depend on Tabard, which
  keeps its own smolweb module for the hosts. Mere's graphshell.git rows moved together to
  `d80316db` (ten packages), since the lanes' `tinct` exists only from there and Cargo cannot
  hold one branch at two revisions offline.

Host follow-up under D16 (not edited here). Each host keeps persisting Tabard's types and
converts at the lanes' boundary:

- Turnstone: 22 uses of `tabard::ReadingAnchor` or `tabard::ReadingPreferences` in 11 files;
  the boundary calls are `src/shell/content_pool.rs` lines 221 and 229
  (`restore_reading_anchor`), 260 and 267 (`reading_preferences()` compared with the host's),
  282 and 289 (`set_reading_preferences`); `src/shell/effects/shell_effect_control_content.rs`
  264 and 268 (`restore_reading_anchor`); `src/shell/reader_observe.rs` 135 and 139
  (`reading_anchor()`) and 188 (`reading_preferences()`); and the test
  `src/shell/content_pool/tests/hidden_finite_livery_work_schedules_ticks_until_settled_cases.rs`
  211 and 215. The persisted anchor stays Tabard's: `src/scope_navigation.rs:26`
  (`Scope.reading`), its deserializer `src/scope_navigation/impl_de_deserialize_de_for_scope.rs`
  13 and 19 (including the legacy shape), `src/reading_view.rs` (the `anchors` map, line 40)
  and `src/action/effect.rs:17` (`RestoreReadingAnchor`).
- Graphshell port (converted, option (a)): the compiler named five sites, all in `ports/graphshell/src/retained_content.rs`
  (299, 305, 306, 313, 315: `reading_anchor`, `restore_reading_anchor`, `reading_preferences`
  over the reader and smolweb sessions); 22 uses of the Tabard types in 6 files stay as they
  are. The persisted anchors are `ports/graphshell/src/forme_workspace.rs` 27
  (`reading_anchors`) and 31 (`scope_navigation`).
- Turnstone's 12 palette sites (`ModeProfile`, `SmolwebTheme`) name the lanes' own types and
  need no edit. Its one meeting point is `src/shell/shell_publish_engine_inventory.rs` 374 to
  384, where `tabard_mode().profile()` (Tinct's `ModeProfile` through Tabard) is assigned to
  the lanes' `color_mode`: the same type once Tabard and Turnstone take `tinct` from
  graphshell.git (D19's repoint), a two-field conversion (`dark`, `high_contrast`) until then.
- Neither host uses `document_style_with_typography`; `tabard-workshop`'s one site is converted
  (`ports/tabard/workshop/src/reader.rs`, `reader_styles`).

Follow-up under D17 (not edited here): Knot's `crates/knot-document/src/editor.rs` takes
`KnotEditor`, `EditOutcome` and `KnotReadout` from `crates/inker/knot-editor-host/src/{model,editor}.rs`
(with `model.rs`'s three tests) and depends on `cambium` and `illume` (feature `markdown`)
instead of `knot-editor-host` (root `Cargo.toml` line 106, `crates/knot-document/Cargo.toml`
line 38); Turnstone's `src/knot_authoring.rs:43` and Mere's `ports/graphshell/src/block_editing.rs:14`
then import `KnotEditor` from `knot-document`, and their manifests drop `knot-editor-host`
(Turnstone `Cargo.toml` line 268). Then `knot-editor-host` retires and leaves Inker.

- F2 (ruled D17, Knot's `knot-document`), `knot-editor-host -> cambium`: stays until Knot
  takes the editor (above).
- F3 (open), `mere-fleece -> mere-rdf` (optional, `structured-data`, no consumer in any manifest).
  It projects JSON-LD blocks into graph contributions, the engine's job. Options: the engine,
  taking plain block text and base address; the host; retirement. Recommended: the engine.
  Finding: `mere-rdf` depends on `mere-kernel` and on `inker`, so Inker cannot depend on the
  engine without a cycle until P5 separates it.
- F4, settled by D19: `script-rhai -> hest` (graphshell.git, a serde-only leaf). Its `command` module turns a
  Rhai batch into `hest::Invocation`s against a host-offered `CommandSet`; Mere's graphshell
  port and Knot's endpoint tests consume it. `hest` is in Graphshell's contract root, which
  Inker may name; nothing changes.
- F5 (settled in Inker 2026-11-07: Inker carries the font with its Apache-2.0 notice), a file edge: `document-canvas`'s text test includes Cambium's
  `examples/genet_web_smoke/assets/Roboto-Regular.ttf` (168 KB, no licence file beside it,
  ten users in Mere). The standalone proof placed a copy at that path in the scratch
  workspace. Options: Inker carries its own copy with the licence, or the family gets one
  test-asset home.

The `fleece-crawl` home in `mere-fetch` is a reversible choice (it lowers `fetch::crawler`);
Mere's graph instantiation of the session, `CrawlSession<GraphId, GraphContribution>`, is
written by the first host that needs it. The branch is rebased onto main after D15 and committed there with the clippy hook run.

### P2. Seiche

Move the runtime crates to `merely-made/seiche`. `numen` and `seiche` keep their declared
dynamics rules and determinism bits (rapier `enhanced-determinism`, the state witness). The
projection crates that consume the workspace kernel stay with the host.
- Done when: Seiche builds and tests alone; Isocosm's Eponym and Mesocosm and Mere's canvas
  resolve it from one source; the dynamics grammar's recorded receipts that involve seiche
  are rerun from the new home with the same figures.
- Needs: P0's lines for the spatial crates.
- State (2026-11-07, worktree only; receipts file): the six crates (`quint-shaders` is gone,
  see P0) check and test green as a standalone workspace on Mere's lock; the seiche binding
  replay rerun from it equals the 2026-10-20 recorded traces witness for witness; Mere's
  canvas checks; Isocosm's Eponym and Mesocosm check against the standalone crates with one
  source each. Not done: the repository, the move, the repins.
- State (2026-11-07, Mere's cut on a worktree branch): Seiche main `a6b8641` holds the seven
  crates (`seiche-model` included) with history and D25's two documents; Mere deleted
  `crates/conatus/`, takes `seiche` and `numen` from `seiche.git` (`pictograph` and
  `graph-kernel` inherit), and redirects both in its `mere.git` patch table until the outside
  consumers repin (Open, After P2's cut). Not done: those repins, and Isocosm resolving
  Eponym and Mesocosm from `seiche.git`.
- Edges the move keeps: `armillary` (family git, optional `actor`), the kiss3d fork by rev,
  and Mere's `cubek-matmul` patch, which the standalone graph lacks (Burn features only).
- Forks, open: (1) `state-witness`, seiche's one edge into Mere, dev-only, for the replay
  examples, and pulling `mere-framing`; while `graph-kernel` and `pictograph` depend on
  `numen` and `seiche`, a Seiche dev edge back into Mere is a mere/seiche cycle under
  cycle-breaks D6. Isocosm's Cosmograph and Isometer also take `state-witness` from
  `mere.git`. Options seen: Seiche depends on it from `mere.git`; it (and `mere-framing`)
  moves to Seiche and Mere consumes it from there; the replay examples move to the host as
  consumer tests. (2) `nisus` into conatus (P0). (3) `graph-kernel`'s `numen` edge: whether
  field truth stays with the engine half (Mere names Seiche) or goes with the workspace
  half to Graphshell.
- The repository will need: the restated root (members, resolver 2, MPL-2.0, the workspace
  lints, `nisus` and `wgpu` rows, the physics dev opt-level table, `rust-toolchain.toml` at
  1.99.1), a lock seeded from Mere's, fork (1) settled, and repins in Mere (`pictograph`,
  `graph-kernel`), Isocosm (Eponym, Mesocosm, Isometer, parry-ground), Cleromancy, Knot,
  Murm's signalman-desktop and mer3ly (which pins `www.github.com/.../mere.git` by rev).

### P3. Graphshell: the host

Reorient the revived repository around the host contract and the hosting (D2): the D9
leaves at the bottom as their own workspace root, the hosting above. Move the Cambium family,
the projections, chrome, registries, the fetch service, `mere:script` and the ports. The
workspace kernel is `incipit` (D8); keep `mere-kernel` resolvable during cutover. Fold the `mere` facade's
navigation, trail and roster logic into the workspace kernel and its view APIs into the view
crates.
- Done when: Graphshell builds the reference host and `mere-resident` assembly with Mere as a
  git dependency and no Mere crate depending back; Turnstone and Pelt run from it with
  today's headed receipts; the layout test's repin count is recorded; the bottom root still
  names no app-level crate.
- Needs: P1 (Inker as a supplier), P2 (Seiche as a supplier), cycle-breaks P6.

### P4. Pandect on the authority boundary

Split `pandect` three ways. The graph-domain revision lifecycle stays as `pandect`: how a
graph-domain operation becomes a durable, reopenable revision (admission, fold, branch,
merge, replay), with Stickleback supplying the log and the domain interface and the engine
supplying assertion semantics. `pandect::reservoir` becomes `catchment`, the reservoir service crate in
Mere (D3, D9): persona and domain catalog, access, store lifetimes, sealing, replication. Sessions,
manifests, view sidecars, worker declarations, settings and furnace paths are traced to
their owners, most to the host; an authored arrangement replicates through the domain log
(contract C7) while Graphshell defines and edits it.
- Done when: a graph edit reopens at the same revision; a non-graph domain (Woodshed's
  `mere/music`, or the Knot store) opens through the same reservoir with no graph or host
  crate in its dependency path; a Graphshell view is rebuilt from the dataset revision it
  cites; each traced sidecar has an owner recorded in P0's lines.
- Needs: P0; the domain contract's C1 to C8 as written.

### P5. The engine crate

Fold `linked-data` and the resource half of `graph-kernel` into one `mere` crate with the
module layout of The engine's stages. The assertion rules come across whole: separate
assertions per asserter, withdrawal retained until compaction, no revival by a stale assert,
reason keys. The old graph-to-RDF projection and the `GraphContribution` import profile go
with the host's legacy snapshot adapter. Oxigraph's component crates stay upstream behind
the engine's dataset and loader interfaces; a fork joins only where upstream cannot take a
change. Measure per Constraints before the resource/surface join becomes a crate boundary.
- Done when: the `mere` crate builds headless on native and wasm with no host, canvas or
  Pandect dependency; the ledger's RDF, SPARQL and JSON-LD rows pass for the declared
  profile at the same or better rate than P0 recorded; load, mutation, projection refresh and
  frame cost are measured against the P0 baseline and recorded; the example headless resource
  inspector runs.
- Needs: P4 (so the profile is outside the crate), P3 (so the surface half has a home).

### P6. Interpretation and identity stages

The pack loader and user-agent pack; RDFS and OWL 2 RL plus pack rules into the derived
layer; SHACL diagnostics; WebID and DID resolution against custody; canonicalization and
proof verification. Each lands behind its ledger rows. The thin slice that proves the
interpretation model: load a resource, select a pinned pack, derive one fact, report one
shape violation, reopen the same revision with the same results.
- Done when: the thin slice passes as a test; SHACL and RDFC-1.0 rows exist with results;
  identity and proof verification have their own rows and gates; the lessons brief's D24 to
  D29 and D17 to D23 each name the module that carries them.
- Needs: P5; the declared-lens plan for the pack format.

### P7. The experience vocabulary as the first external pack

Traversal, arrangement and containment-as-layout become Graphshell's pack over the engine:
the engine carries the terms, their stratum and the axes it observes, and has no behavior
for them; the host owns the behavior. A layout grouping exports as RDF while its editing
rules stay in the workspace.
- Done when: the pack loads through the common loader; a trail and an arrangement round-trip
  through the engine's export and reopen unchanged; no engine code names a workspace term.
- Needs: P5, P6's loader.

### C. Consolidations (D7)

Three repositories fold into their consumers, each as a workspace member of the receiver
with history preserved as the 2026-10-19 promotion did. `smolweb` (the protocol family:
`errand`, `gemini-protocol`, `gopher-protocol` and the rest) into Inker with P1, since
`nematic` and `inker` are its consumers; `misfin` is the exception to check, since
`mere-comms` is its other consumer and Murm may be its home. `emblem` (IconVG) into
Graphshell with P3, since `pictograph` is its only consumer. `wavicle` (WavPack) into
Woodshed, its only consumer, on Woodshed's own schedule. The four `wgpu-*` repositories
stay (D7, D21), with Inker recorded as the owner of scrying and welding. Inside each wgpu
repository the audit's folds apply: the `grafting-servo` rename finished,
the scry and weld demo crates folded, the three `keyutils.rs` copies deduplicated.
- Done when: each folded repository is archived with a pointer to its new home; its crates
  publish from the receiver under their existing names; every consumer resolves them from
  one source; the family's repository count is recorded.
- Needs: P1 for smolweb, P3 for emblem; nothing in this plan for wavicle.

### P8. Cutover and the engine-to-workspace contract

Turnstone and Knot cut over from `mere.git` workspace and reading pins to Graphshell and
Inker sources, old revisions kept available. Then, with the pieces in their homes, settle the
engine-to-workspace contract Mark deferred (D4): how a host binds a view to an exact dataset
revision and what crosses at frame rate.
- Done when: neither product pins `mere.git` for anything but the engine; the contract is a
  dated design document with its own rulings; the three probes from the second layout (a
  headless assertion edit and reopen, a host view bound to that revision, an arrangement edit
  leaving the revision alone) pass as tests.
- Needs: P1 to P5.

## Decisions

- **D1. The split (Mark, 2026-11-07):** "I agree with the overall split", and "me personally,
  i think splitting is ok. pandect was becoming another grab bag like djinn anyway. we should
  keep an eye out for crates that have drifted from a clear intention, too.
  redefining/refactoring them is possible, but i agree that planning now beats not having a
  plan later." Mere becomes the engine and the profile; the host, reading and spatial stacks
  leave; every touched crate states one intention. Outcome: refines Stack 3, whose "Mere
  consumes producers" and "graph-owning producers compile in lockstep with it" name the host
  runtime, which is now Graphshell; the statement's seams and formats are unchanged.

  Links: area The stack · amends mere:stack.3 · draws on linked-data-platform-lessons D2, linked-data-platform-lessons D31 · plans engine-split P0, engine-split P5 · summary Mere is the semantic-web engine and the person's profile of it; the host, reading and spatial stacks leave; each crate keeps one intention.

- **D2. Graphshell is the host, contract and hosting together (Mark, 2026-11-07):** asked where
  the workspace host lives given cycle-breaks D8, against a new repository and against
  staying in Mere: "eh, this would reorient the dependency graph anyway. reorienting
  graphshell around the host contract makes more sense than just putting a wire there and
  leaving the hosting on top. otherwise, we'll be making a cambium repo, which i'd rather
  avoid making more repos unless there's a really good reason. indeed,
  consolidating/removing repos is a priority for me: once we get above like ~30, my brain
  starts to fizz and pop. 25 is better, 20 moreso." The revived repository holds the D9
  leaves as its bottom root and the hosting above. Outcome: refines cycle-breaks D8
  ("graphshell = the protocol") and D9; the repository count becomes a constraint.

  Links: area The stack · amends cycle-breaks D8, cycle-breaks D9 · draws on mere:stack.3 · plans engine-split P3 · summary The Graphshell repository holds the host contract and the hosting; repositories are a cost to be paid down.

- **D3. The reservoir service lives in Mere, outside the engine (Mark, 2026-11-07):** "Mere
  repo, own crate, outside the engine (Recommended)". Pandect splits three ways; the
  Woodshed check is the done-condition. Outcome: applies adoption D8 and contract C5.

  Links: area The stack · draws on domain-adoption D8, domain-contract C5 · plans engine-split P4 · summary The reservoir service is its own crate in the Mere repository; Pandect keeps the graph-domain revision lifecycle.

- **D4. Moves do not wait on the engine contract (Mark, 2026-11-07):** asked what must be
  true before the first move: "inker and the physics can move. the hosting can move. i don't
  mind figuring out the contract after things are clearer". Outcome: new; sets the phase
  order and sends the contract to P8.

  Links: area The stack · plans engine-split P1, engine-split P2, engine-split P3, engine-split P8 · summary Inker, Conatus and the hosting move first; the engine-to-workspace contract is settled after.

- **D5. The spatial split is the runtime only (Mark, 2026-11-07):** "Runtime only: conatus,
  modulus, nisus, tenant, numen, seiche (Recommended)". The projection crates stay with the
  host. Outcome: applies Stack 3 (scene crates published, hosts consume them).

  Links: area The stack · draws on mere:stack.3 · plans engine-split P2 · summary Seiche takes the spatial runtime; the kernel-consuming projections stay with the host.

- **D6. The spatial repository is named seiche (Mark, 2026-11-07):** "if i was gonna pick a
  name to foreground for the spatial repo, i like seiche. what can i say, it's a cool name."
  The published crate keeps its name and the repository takes it; no new word is spent.
  Outcome: applies D5.

  Links: area The stack · draws on engine-split D5 · plans engine-split P2 · summary The spatial runtime's repository is merely-made/seiche.

- **D7. Consolidations (Mark, 2026-11-07):** "the consolidation targets are reasonable. i
  think the wgpu ones probably should stay separate but smolweb (inker?), emblem (inker?
  graphshell?), and wavicle (woodshed?) are good picks." Read against the consumers: smolweb's
  consumers are `nematic` and `inker`, so Inker; emblem's only consumer is `pictograph`, so
  Graphshell; wavicle's only consumer is Woodshed. The `wgpu-*` repositories stay. Outcome:
  applies D2's repository-count constraint; the family nets 24 repositories from 25.

  Links: area The stack · draws on engine-split D2 · plans engine-split C · summary smolweb folds into Inker, emblem into Graphshell, wavicle into Woodshed; the wgpu repositories stay separate.

- **D8. The workspace kernel is incipit (Mark, 2026-11-07):** asked whether a name could be
  reclaimed from a crate due for reorganisation rather than spent: "Namewise, frisket or
  incipit feel right to me. Incipit in particular". `incipit` is already the family's
  "workspace identity vocabulary" (298 lines, published 0.2.1, depended on by the `mere`
  root, `mere-view`, `fleece`, Turnstone and Cleromancy, none in the protocol layer); the
  kernel grows from it, and the ids stay a minimal default feature. `frisket` stays a
  name claim with no code (`genet/support/name-claims`). Outcome: applies D1.

  Links: area The stack · draws on engine-split D1 · plans engine-split P3 · summary The workspace kernel is incipit, grown from the ids crate with the ids kept behind a minimal feature.

- **D9. The reservoir service crate is catchment (Mark, 2026-11-07):** "catchment
  (Recommended)", `reservoir` being taken on crates.io (a sampling crate). Checked free on
  crates.io on 2026-11-07; claimed when the crate publishes. Outcome: applies D3.

  Links: area The stack · draws on engine-split D3 · plans engine-split P4 · summary The reservoir service crate is catchment.

- **D10. Squish the sprawl; free names before spending them (Mark, 2026-11-07):** "Ok, look, if
  the sprawl can be squished in a practically advantageous way, that's valuable. I'd rather
  free names for actually distinct purposes than make new ones". Merging becomes P0's default
  disposition where a crate's job is one module of a neighbour's, and a freed family name is
  taken before a new word. Outcome: applies Stack 1 ("a component is a module in its parent
  crate, not a crate of its own"); refines the naming ledger's rules.

  Links: area The stack · draws on mere:stack.1, engine-split D1 · plans engine-split P0 · summary Crates whose job is a neighbour's module merge into it; a distinct purpose takes a freed family name before a new word is spent.

- **D11. Sweep the whole tree (Mark, 2026-11-07):** asked whether to commit the split-plan files
  alone, sweep, or wait, since both lanes' branches carry their own copies of the uncommitted
  plan: "Sweep the whole tree". One commit on main carrying everything dirty, its message
  naming what it swept. Outcome: operational; recorded here only because the lanes' merges
  depend on it.

  Links: area The stack · plans engine-split P8 · summary Main commits the whole dirty tree so the lanes' plan copies can merge.

- **D12. state-witness and mere-framing go to Eidetic (Mark, 2026-11-07):** P2's blocking fork,
  seiche's dev-edge into Mere being a mere/seiche cycle under cycle-breaks D6 and Isocosm a
  direct consumer: "Eidetic (Recommended)". Both crates move below Mere and Seiche; Isocosm,
  Mere and Seiche repin to eidetic.git. Settles the dynamics grammar plan's open question on
  seiche taking state-witness, and audit fork F9. Outcome: new; applies cycle-breaks D6.

  Links: area The stack · draws on cycle-breaks D6, engine-split D5 · plans engine-split P2 · summary state-witness and mere-framing move to Eidetic; Seiche and Mere consume them from there.

- **D13. The canvas takes the cartography name; pictograph shrinks to faces (Mark, 2026-11-07):**
  audit fork F1, pictograph being 93% graph canvas and mere-cartography holding six layout
  strategies: "Canvas plus strategies take the cartography name; pictograph shrinks to faces
  (Recommended)". The 63.8k canvas lines and cartography's strategies and contracts become one
  crate under `mere-cartography`; `pictograph` keeps the 4.4k lines its name describes. No name
  freed; two used correctly. Outcome: applies D10. Timing (Mark, 2026-11-08, with another
  session making 11 pictograph commits in a day): "Fold it into P3"; the rename happens at the canvas's
  move to Graphshell, one move instead of two.

  Links: area The stack · draws on engine-split D10 · plans engine-split P3 · summary The graph canvas and the layout strategies are one crate named mere-cartography; pictograph is node faces only.

- **D14. Retire the five dead registries now (Mark, 2026-11-07):** audit fork F17, 3,876 lines
  with no consumer in six repositories: "Retire now (Recommended)". `input`, `knowledge`,
  `layout`, `lens` and `protocol` are deleted; git keeps them; `mod-loader`, `observations` and
  `diagnostics` move to their owners and `mere-registry` is freed. Outcome: applies D10. Refined by D26: the crate keeps its name as
  the diagnostics and observations library.

  Links: area The stack · draws on engine-split D10 · plans engine-split P3 · summary The five never-enabled registries are deleted; the three live ones move to their owners; mere-registry is freed.

- **D15. Repin Knot before P1 lands (Mark, 2026-11-07):** the pre-commit clippy fails on main
  before any P1 change, Knot's locked `knot-composition` still using `scenomise::backdrops`
  after cycle-breaks D17 moved it: "Repin Knot first (Recommended)", over committing P1 with
  the hook skipped. A Knot repin lane updates `knot-composition`, Mere relocks, and P1 commits
  normally. Outcome: operational; it orders P1 and unblocks every Rust commit on main.

  Links: area The stack · draws on cycle-breaks D17 · plans engine-split P1 · summary Knot is repinned to the moved backdrops and Mere relocked before P1 commits; the hook is not skipped.

- **D16. The reader lanes own their types (Mark, 2026-11-07):** P1 fork F1, the lanes' API taking
  Tabard's `ReadingPreferences` and `ReadingAnchor`, Turnstone using them in forty-odd places and
  persisting the anchor: "The lanes own their types; hosts convert at the edge (Recommended)".
  Inker gains no Tabard dependency; Turnstone and the Graphshell port convert at the call
  and at anchor load. Outcome: applies cycle-breaks "cut first".

  Links: area The stack · draws on engine-split D1 · plans engine-split P1 · summary mere-lanes owns ReadingPreferences and ReadingAnchor; hosts convert Tabard's settings at the edge.

- **D17. The Knot editor lives in knot-document (Mark, 2026-11-07):** P1 fork F2, `KnotEditor`
  wrapping Cambium's `TextInput` with three importers: "Knot's knot-document (Recommended)".
  Knot owns its editor and takes Cambium for it; the lexers go to Illume; `knot-editor-host`
  retires and its name is freed. Outcome: applies D10.

  Links: area The stack · draws on engine-split D10 · plans engine-split P1 · summary KnotEditor moves to Knot's knot-document; the lexers to Illume; knot-editor-host retires.

- **D18. Create both repositories now, empty (Mark, 2026-11-07):** "Create both now, empty;
  lanes populate once clean (Recommended)". `merely-made/inker` and `merely-made/seiche` were
  created public the same day, with nothing pushed. Outcome: operational.

  Links: area The stack · plans engine-split P1, engine-split P2 · summary The inker and seiche repositories are created empty under merely-made; their lanes populate them once clean.

- **D19. Inker may name the bottom root; tinct joins it (Mark, 2026-11-07):** Tinct's ten
  consumers in six repositories all sit beside Cambium or Tabard, Tinct depends on serde only,
  and P1's F4 (`script-rhai` naming `hest`) and F6 (the smolweb palette deriving through Tinct)
  are the same question: "Yes: tinct joins the bottom root, Inker may name it (Recommended)".
  `tinct` moves into the Graphshell repository's Mere-free contract root (cycle-breaks D9's
  leaves); Inker takes `tinct` and `hest` from that root, which names nothing above it; the
  palette moves into the lanes over Tinct and `script-rhai` keeps its `command` module.
  Outcome: refines cycle-breaks D9 (the root gains tinct) and settles P1's F4 and F6. Cost
  named on 2026-11-07: the contract root drifts from "the host/guest contract" toward "the
  family's leaves" (hest, now tinct), tolerable while it names nothing above it, so the next
  leaf is a decision, not a habit; and Inker naming graphshell.git while the Graphshell host
  names Inker is a repository-level back edge with no package cycle, which fixes the lockfile
  order as bottom root, then Inker, then host.

  Links: area The stack · amends cycle-breaks D9 · draws on engine-split D2, engine-split D16 · plans engine-split P1, engine-split P3 · summary The Graphshell contract root gains tinct and is a leaf layer Inker may name; the palette lives in the lanes over Tinct.

- **D20. scrying and welding move into Inker; cue and graft stay (Mark, 2026-11-07):** on the
  producer/host reading of the wgpu family (scry and weld drive the OS webview and CEF and have
  no consumer but Inker's adapters; cue is the host's embedding contract and graft its GPU
  importer, pinned by Pelt, Graphshell, Mere and Turnstone): "scry and weld into Inker as
  feature-gated members; cue and graft stay (Recommended)". CEF and WebKit build weight is
  opt-in behind features. Mark, on the difference: "huh. odd. wonder why cue and graft are
  different than the other two on this count." Outcome: refines D7 (the wgpu repositories no
  longer all stay); the family nets 22 repositories from 25.

  Links: area The stack · amends engine-split D7 · draws on mere:stack.3 · plans engine-split C · summary wgpu-scry and wgpu-weld fold into Inker as feature-gated engine members; wgpu-cue and wgpu-graft stay below Inker.

- **D21. D20 reverted: scrying and welding stay separate, Inker named as their owner (Mark,
  2026-11-07):** on the pushback that a Cargo workspace member cannot be feature-gated (it is
  always resolved, and `cargo check --workspace` builds it), so welding's `cef-dll-sys` build
  script, which downloads the CEF bundle, and scrying's WebKitGTK and WPE system libraries
  would land on every machine that touches the reader, and two published crates at 0.7 and
  0.15 would be tied to Inker's cadence, for two fewer repositories: "Revert D20: scry and
  weld stay separate, Inker named as their owner (Recommended)". D7 stands for all four wgpu
  repositories; Inker's docs list scrying and welding as the engines it orchestrates; the
  audit's demo folds and the `grafting-servo` rename happen inside each. The family nets 24
  repositories from 25. Outcome: replaces D20; applies D7.

  Links: area The stack · replaces engine-split D20 · draws on engine-split D7 · plans engine-split C · summary The wgpu repositories all stay; Inker is recorded as the owner of the scrying and welding engines.

- **D22. An eighth Stack statement, for the roots (Mark, 2026-11-07):** asked what the trunk is
  if it is not Mere, Graphshell, Inker or Genet, and whether leaves are guests: "i will give one
  more statement for a core architectural concern. 8?" The answer given: leaves are not guests
  (a guest implements the contract; a leaf is a library both sides link); the graph has two
  roots by nature, a data root (Eidetic: logs, stores, the domain interface, the pack header,
  the witness and the frame) and a presentation root (the contract workspace: projection
  contract, WIT world, scene types, command vocabulary, colour), with custody beside them;
  a leaf goes to the root of its nature; one trunk is not forced. Outcome: new, Stack 9; the
  Stack budget rises to eight.

  Links: area The stack · draws on engine-split D12, engine-split D19, domain-contract C5 · summary The stack has a data root and a presentation root; a leaf goes to the root of its nature; Stack 9 says so.

- **D23. Graphshell is the host layer as a library; Turnstone takes the reference-host duties
  (Mark, 2026-11-07):** asked whether to make Graphshell infrastructure with Turnstone taking
  its duties, or expand Cambium and make Graphshell only a host: "Graphshell is the host layer
  as a library; Turnstone takes the reference-host duties (Recommended)". The repository keeps
  the contract root below and the toolkit, kernel, projections and resident assembly above,
  with no application; the reference host application, its loopback acceptance view and the
  MCP server move to Turnstone, which already composes Pelt and Graphshell. The toolkit's home
  is fixed by its consumers (Knot's desktop, Woodshed, Cleromancy and Isocosm pin Cambium,
  Tabard and mere-view), so it cannot live in a product repository. Outcome: refines D2 and
  cycle-breaks D8 ("graphshell = the protocol" becomes "graphshell = the host layer"); the
  name stops meaning an application.

  Links: area The stack · amends engine-split D2, cycle-breaks D8 · draws on mere:stack.3 · plans engine-split P3 · summary The Graphshell repository is the host layer as a library, contract root below and hosting above, with no application; Turnstone holds the reference host and the MCP server.

- **D24. Fleece's fixtures copy into the Graphshell port (Mark, 2026-11-07):** when Mere deletes
  `crates/fleece`, the port's tests and fixture example read Fleece's portable-clip fixtures
  (`knot.json`, `redshank.json`, `voice.wav`) by relative path in eight places: "Copy the three
  fixtures into the port (Recommended)", over reading them from Inker's checkout or a shared
  test-asset home (the family-wide answer F5 also raised). Outcome: applies P1; the shared
  asset home stays an open fork.

  Links: area The stack · draws on engine-split D1 · plans engine-split P1 · summary The Graphshell port carries its own copies of Fleece's three portable-clip fixtures.

- **D25. The conatus engine plan and its two briefs move to Seiche as its first area (Mark,
  2026-11-07):** they sat in `mere_docs/` with no area root, so by the letter they stayed, while
  their subject is now entirely Seiche: "Move the conatus plan and its two briefs to Seiche as its
  first area (Recommended)". Mere's addendum rule, when code moves out its docs move with it. The
  dynamics grammar and physics catalog plans have consumers on both sides and stay in Mere, cited
  from both. Outcome: applies DOC_POLICY §4 (docs live with the owner of the subject).

  Links: area The stack · draws on engine-split D5 · plans engine-split P2 · summary Seiche's first design area holds the conatus engine plan and its two briefs; the dynamics grammar and physics catalog plans stay in Mere.

- **D26. Diagnostics stays a library; mere-registry keeps its name as that library (Mark,
  2026-11-08):** D14 met a cycle in the lane: `mod_loader` (2,385 lines) calls `diagnostics`'
  `emit_event`, `DiagnosticEvent` and eight `CHANNEL_MOD_*` constants, Turnstone names
  `mere-registry` for `diagnostics` alone, and `diagnostics` is 4,346 lines with its
  subdirectories, not the 956 the audit counted. Moving it to the resident would make
  `document-host` and Turnstone depend on a port. Offered a module of `document-host` or a small
  crate under the freed name: "either 1 or 2 are ok with me, whatever makes life easier". The
  easier is the crate: `mere-registry` stays, trimmed to `diagnostics` and `observations`; the five
  dead registries are deleted; `mod_loader` moves to `document-host`; no consumer row changes.
  Outcome: refines D14 (the name is not freed; the retirement of the five and the loader's move
  stand). The same round confirmed two lane choices under D10: `incipit` 0.2.2 rather than 0.3.0
  (Distillery needs `^0.2`) and `persona_picker` behind an optional `cambium` feature (its
  dependencies add about 160 identity and crypto crates to a crate 23 manifests depend on):
  "Both stand".

  Links: area The stack · draws on engine-split D10, engine-split D14 · plans engine-split P0 · summary mere-registry stays as the diagnostics and observations library; the five dead registries go and mod_loader moves to document-host; incipit bumps compatibly and persona_picker is a cambium feature.

- **D27. backstep grows into the editor's ephemeral state (Mark, 2026-11-08):** P0 fork F4, the
  `cambium` merge being a cycle and `meristem` the only receiver left: "If undo history can be diff,
  merge it. If not, keep it and consider what we might do with it. Do we want to add code from, say,
  arboard or something in there? Are there relevant utility states/commands? Compass? Idk, throwing
  spaghetti here". Evidence: meristem diffs retained view trees for a renderer; backstep records
  whole-document snapshots before each edit and rules out inverse commands, with durable undo in the
  session journal, so undo cannot ride that diff. No `arboard` in Mere; clipboard is a host operation
  refused at four sites (Cambium's text input, rootstock's egui and iced embeddings, chrome's frame
  model, the Graphshell web share); navigation history exists three times with three subjects
  (chrome's linear back/forward, 383 lines; the kernel's shared browse trail, 488; the port's
  checkpoints, 319); nothing is named compass. Put back as keep, grow with a clipboard contract, or
  grow with the contract and chrome's back/forward: "Grow: undo, clipboard contract, and chrome's
  back/forward". `backstep` stays dependency-free: it gains a host-implemented clipboard trait the
  four sites code against (hosts bind arboard or the browser API) and chrome's linear history as its
  second ephemeral history; the kernel's trail and the port's checkpoints stay. Outcome: refines D10
  (a crate may grow by taking its neighbours' scattered copies of its own job).

  Links: area The stack · draws on engine-split D10 · plans engine-split P0 · summary backstep is the editor's ephemeral state: snapshot undo, a host-implemented clipboard contract, and chrome's linear back/forward; it stays dependency-free.

- **D28. forme merges into incipit behind an arrangement feature (Mark, 2026-11-08):** P0 fork
  F5: "Compare 1 and 2. 2 seems cleaner, should incipit grow its ambit?" Evidence: forme is
  framework-free (serde, incipit, serde_json, taffy, optional petgraph, uuid; the receipt's kernel
  dependency is a dev-dependency and its manifest's "mere-kernel depends on mere-forme" comment is
  stale); platen carries kernel, backstep, inker, document-canvas, workbench, uxtree and accesskit,
  and forme's other dependents `mere-chrome` and `pictograph` take only forme, so into platen hands
  them the reading stack for a data type; into incipit costs a feature carrying taffy and petgraph
  with the default still serde and uuid, which is the growth D8 already rules. "Into incipit behind
  an arrangement feature (Recommended)". Frees `mere-forme`. Outcome: applies D8 and D10.

  Links: area The stack · draws on engine-split D8, engine-split D10 · plans engine-split P0 · summary forme's arrangement authority is incipit's arrangement feature; incipit's default stays ids and curation; mere-forme is freed.

## Open questions

- **P0 forks F1 to F25** are listed with their evidence in the receipt file's audit section.
  The four that change assigned moves are ruled: F1 by D13 (folded into P3), F4 by D27, F5 by D28,
  F17 by D14 and D26; the rest follow in later rounds. F2 is the
  engine's own: `linked-data/src/statements.rs` makes the engine depend on `inker`, which a
  2026-07-10 ruling placed there so Inker stayed kernel-free; under the split it belongs with
  Inker's emitted statements or the host's intake adapter.
- **P2 forks (Seiche lane).** `state-witness` (settled by D12; Seiche takes it from eidetic.git): seiche's dev-edge back into Mere is a
  mere/seiche cycle under cycle-breaks D6, and Isocosm consumes it directly; Seiche takes it
  from `mere.git`, or it and `mere-framing` move to Seiche, or the replay examples move to
  the host as consumer tests. `nisus` into `conatus` behind a voxel feature would free the
  name, but Mesocosm takes nisus separately to stay free of a physics backend.
  `graph-kernel`'s numen edge: field truth stays with the engine half (Mere names Seiche) or
  goes with the workspace half to Graphshell.
- **P1 forks (Inker lane; evidence in its P1 block and the receipt file).** F1 reader styles:
  the lanes own their own `ReadingPreferences` and `ReadingAnchor` and each host converts
  Tabard's at the edge (recommended), or the types move to the lanes with Tabard re-exporting,
  or they go to `inker`. F2 `knot-editor-host`: `KnotEditor` wraps Cambium's `TextInput` and
  three consumers import it; home is Knot's `knot-document` (recommended), `cambium` behind a
  feature, a host crate, or a buffer trait; the lexers go to Illume either way. F3 Fleece's
  `structured-data` feature turns JSON-LD blocks into graph contributions with no consumer:
  to the engine (recommended), the host, or retired; Inker cannot depend on `mere-rdf` until
  P5 without a cycle. F4 `script-rhai`'s `command` module builds `hest` invocations: to the host
  (recommended), or `hest` ruled a contract leaf Inker may name. F5 (settled: Inker carries its own) `document-canvas`'s text
  test reads Cambium's bundled Roboto font with no licence file: Inker carries its own with a
  licence, or the family gets one home for test assets. Also: the repository's pre-commit
  clippy fails on main before any P1 change (Knot's locked `knot-composition` still uses
  `scenomise::backdrops`, moved out by cycle-breaks D17): commit with the hook skipped, or
  repin Knot first.
- **Eidetic squish (read-only audit, 2026-11-07, in the receipt file).** Candidates by names
  freed per line: `mere-proofs` into `muniment` (161 lines; gemot's proofs-only manifests need
  a feature); `hagiograph` with `impresa` (both zero-dependency isocosm organs); `eidetic-search`
  into `eidetic` behind a feature; `servitor` into `chartulary`. Fork F4 there is the one that
  matters for the engine: about 7,700 lines of `eidetic-core` (57%) are Mere-product code
  (`browsing`, `import`, `models`, `records`, `content_store`, `deleted`, `clip_profile`,
  `image_content`) that stay, move to a Mere-side owner, or split at the adapter leaving a
  lean `eidetic` of schema, codicil, typed, seal, retention and pack. C5's domain interface is
  a `domain` module in `stickleback` (policy and authority hooks exist; grammar and fold traits
  do not); D10's pack header splits out of `PackManifest` in `eidetic::pack`. No new crate
  either way.
- **D12 and D19 consumers.** Mere's side is done (Progress, 2026-11-07): the three crates are
  deleted, `pictograph` and `seiche` take `state-witness` from eidetic.git, the root `framing`
  row is gone, `tinct` and `tincture` come from graphshell.git and the `tinct` patch row is
  dropped. Outside Mere, done by 2026-11-08: Isocosm (`753d9702`) repinned `state-witness` to
  eidetic.git at `shared/isometer/crates/isometer-core/Cargo.toml:21` and
  `shared/cosmograph/Cargo.toml:31`; the five tinct consumers repointed their own `tinct` row to
  graphshell.git in the relock that moved their mere.git past this change (Cleromancy `e79576b`,
  Isocosm `753d9702`, Knot `189487c1`, Woodshed `648ae565` across its root, redshank and hocket
  rows; mer3ly `47599d6` had no row and only relocked). Still open: Turnstone's `tinct` redirect,
  which leaves now that Woodshed has repinned (in flight). Seiche's replay example takes the witness from eidetic.git
  (Seiche `a6b8641`), and with Mere's copies deleted (P2's cut) the mere/seiche cycle is gone. Audit fork F9 (merge framing into the witness)
  would be clean in code but not in metadata: framing publishes at 0.0.2 as the data-formats
  rule's universal header, the witness does not publish; both names stay.
- **P1 forks F4 and F6: settled by D19.** `tinct` is in the Graphshell contract root and Mere
  takes it from there; the lanes may take it now.
- **P1 fork F6, the smolweb palette (new, 2026-11-07, now settled by D19; kept for the record).** `SmolwebTheme::Site` derives each site's
  palette through Tinct at construction and on every colour-mode change, and `ModeProfile` is
  Tinct's type in the lanes' public API (Turnstone names it 12 times). Options: the lanes own
  the theme, the palette and a two-flag colour mode and the host supplies the per-site palette
  through a resolver carried by `Site` (recommended, the shape of D16); rule Tinct, a
  serde-only colour library, a leaf Inker may name and move the palette over it; or move Tinct
  to Inker.
- **Host follow-ups under D16 and D17 (recorded by the P1 lane, with file paths in its P1
  block).** Turnstone converts at 13 boundary calls in `content_pool.rs`,
  `shell_effect_control_content.rs` and `reader_observe.rs` and keeps persisting Tabard's
  anchor; the Graphshell port converts at 5 sites in `retained_content.rs`. Done on
  2026-11-07: Knot's `knot-document` holds `KnotEditor` in `retained` (Knot `a58ba28`) and
  the port imports it from there. Open: Turnstone's `src/knot_authoring.rs` and its
  `knot-editor-host` row; with that, `knot-editor-host` has no consumer and retires from
  Inker. `tabard-workshop` was converted by the lane already.
- **After P1's cut (2026-11-07).** Knot at `2c703c4` named `script-rhai`, `illume`, `inker`,
  `knot-editor-host` and `nematic` from `mere.git`; Knot `a58ba28` takes them from
  `inker.git`, and Mere's five redirect rows left with its relock (Progress, 2026-11-07).
  Knot still names `seiche` through `mere.git` (its `Cargo.toml` 79), so the two Seiche
  redirects stay. Turnstone repins its own Inker rows to `inker.git`. Inker
  names `cambium` (and through it the Cambium family) and `mere-rdf` from `mere.git`, which
  every consumer must redirect to one source; the host's half of F2 ends that edge. Active
  Mere documents still carry 28 code-font paths into the moved crates in 16 documents (the
  doc audit's `missing_known_root_paths`, 428 before the cut and 434 after); Markdown links
  were repointed, these were not. `graft-engine` and `weld-engine` have no Mere consumer,
  so they left the lock with `welding` 0.15.0; their two workspace rows stay for the
  hosts that inherit them. `.cargo/config.toml.example` lines 71 and 72 still describe
  `knot-editor-host` as a workspace member.
- **After P2's cut (2026-11-07).** Rows that name the spatial crates through `mere.git` and
  repoint to `git = "https://github.com/merely-made/seiche.git", branch = "main"`, checked
  against each repository's current tree: Isocosm `eponym/Cargo.toml` 54 (`conatus`), 55
  (`modulus`), 57 (`tenant`); `mesocosm/Cargo.toml` 29 (`tenant`), 37 (`nisus`), 46
  (`conatus`); `shared/isometer/crates/isometer-lens/Cargo.toml` 26 (`modulus`), 44
  (`conatus`); `mesocosm/crates/probes/parry-ground/Cargo.toml` 17 (`conatus`); and, to
  eidetic.git under D12, `state-witness` at `shared/isometer/crates/isometer-core/Cargo.toml` 21
  and `shared/cosmograph/Cargo.toml` 31. Cleromancy `Cargo.toml` 74 and `core/Cargo.toml` 40
  (`seiche`, serde, no defaults); Knot `Cargo.toml` 79 (`seiche` 0.0.6, serde, no defaults);
  Murm `apps/signalman-desktop/Cargo.toml` 67 (`seiche`); mer3ly
  `crates/repo-graph/Cargo.toml` 27 (`seiche`, now `branch = "main"`, no longer the
  `www.github.com` rev pin P2's block records). Isocosm's `.cargo/tabletop-local.toml` 62 and 63
  overlay `numen` and `seiche` onto Mere's deleted paths. Done 2026-11-08: Cleromancy (`e79576b`),
  Knot (`189487c1`), Murm (`df3e293`) and mer3ly (`47599d6`) repinned these rows, and Mere's
  `numen` redirect left, then its `seiche` row once Mere relocked Cleromancy to `d9fc819` the
  same day; nothing is pending here.
  Active Mere documents still carry 21 code-font lines into `crates/conatus/` in 10 documents;
  the one Markdown link was repointed.
- **Knot repin (D15) on the branch.** Four of nine tests in `knot-editor`'s
  `composition_retention` fail on Windows with `CheckpointStorage("legacy checkpoint storage
  replacement is supported only on Unix")`, from the `#[cfg(not(unix))]` arm in
  `checkpoint_migration.rs:328-331` added in `16a4b48` on 2026-10-22; pre-existing by code
  evidence, not rerun on the old lock. Settle on a Unix machine or with the old lock.
- **wgpu-* squish (read-only audit, 2026-11-07, in the receipt file).** The library crates are one
  intention each; the squish is demos and a rename: finish the `grafting-servo` rename (the
  directory and lib name still say `servo-wgpu-interop-adapter`; Turnstone's `shell/servo.rs`
  names the lib); fold wgpu-scry's `demo-linux6`, `demo-wpe` and `demo-scrying-winit` into
  `demo-linux` as bins with `required-features` (frees three names); fold the three
  `demo-weld-*` crates (two names; the Windows cdylib complicates it); dedupe three
  byte-identical `keyutils.rs` files into `demo-support` (662 lines, no name). Forks there:
  `scrying::native_frame` and `welding::native_frame` each re-declare grafting's
  `InteropBackend`, `NativeFrame` and sync types (about 3.5k duplicated lines, differing
  `DmaBufPlane` shapes), and scrying and welding share `NavigationEvent`, `Cookie`,
  `CursorShape`, `DragInput` and `KeyEvent`: shared vocabulary, not a repository merge (D7).
- **Misfin's home (C).** It sits in smolweb but serves `mere-comms`, which cycle-breaks D4
  sends to Murm. Whether it follows smolweb into Inker or goes to Murm.
- **Pandect's traced parts (P4).** Owners for settings, view sidecars, worker declarations and
  the furnace paths, found in P0.
- **The derived layer's engine (P6).** Lessons D29 gates differential-dataflow on a wasm
  probe; if it fails, the fixpoint loop over `spareval` is the fallback.
- **Which Oxigraph pieces, if any, need a fork** for SPARQL 1.2 or the dataset trait (P5).
- **The frame-cost budget** the P5 measurement is judged against; the dynamics grammar's
  settle and frame figures are the nearest existing numbers.
- **Doctrine wording.** Stack 3 is reworded minimally under D1; Surfaces 1 to 4 name
  "a product" and "a host" and need no change. The stance doc rewrite waits for P5.

## Findings

- 2026-11-07: Mere holds 62 non-probe crates in 20 families and 10 ports (`Cargo.toml`,
  `crates/*/`, `ports/`); `crates/crawl` and `crates/import` are empty. `Code/crates` holds 16
  forks, none of which belong in the engine; `wpt` is the model for vendoring the W3C suites.
- 2026-11-07: cycle-breaks D8 and D9 (2026-11-06) had made the revived Graphshell repository
  the protocol-only bottom layer; both independent layouts of the same day put the host there.
  Put to Mark; D2 resolves it.
- 2026-11-07: `pandect` carries three jobs (reservoir, graph-domain lifecycle, sessions and
  sidecars); `mere-fetch` carries persona cookies, a Pandect dependency and Inker engine-flip
  state (`crates/system/fetch/Cargo.toml`); the `mere` facade mixes navigation, trail and
  roster with visual and workbench APIs (`crates/mere/src/lib.rs`); `mere-registry` holds
  eight registries of different owners. These are P0's named seams.
- 2026-11-07: `seiche` is kernel-free by its own description; `pictograph` and
  `mere-cartography` depend on the workspace kernel; Isocosm's Eponym and Mesocosm consume
  `conatus` without the graph. This is D5's evidence.
- 2026-11-07: host-side crates measured (lines of Rust under `src`, in-tree dependents across
  Mere, Turnstone, Knot, Woodshed, Cleromancy and Isocosm manifests; counts include a stale
  `.claude/worktrees` checkout and are upper bounds): `mere-kernel` 51.2k/9, `pandect` 27.1k/19,
  `mere-registry` 11.4k/3, `mere` facade 6.9k/18, `mere-forme` 6.7k/2, `mandrake` 6.2k/21,
  `mere-view` 6.1k/10, `mere-cartography` 5.4k/3, `platen` 4.8k/9, `mere-overmap` 4.1k/12,
  `sprigging` 3.0k/33, `mere-chrome` 2.7k/3, `ux-events` 2.1k/2, `uxtree` 1.6k/10,
  `settings-wit` 1.4k/4, `mere-curation` 0.6k/7, `incipit` 0.3k/5, `backstep` 0.3k/19,
  `persona-picker` 0.3k/5. Reading for P0: the projection pipeline has four names for one
  flow (curation, forme, cartography, platen); forme's arrangement authority and curation's
  subgraph specs are workspace truth, which is incipit's job, and cartography's contracts have
  one implementer, pictograph. Platen and uxtree are single-job and stay whole.
- 2026-11-07, P0 audit (receipt file, spot-checked): `pandect` is about 6% reservoir service,
  61% revision lifecycle (3.2k of it tests) and 32% sessions, sidecars, settings and furnace;
  replication is not in it but in the resident's `personal_sync`. `pandect` depends on
  `tabard`. `mere-fetch` has no RDF resolution policy today, only a caller-supplied accept
  argument, so the engine's loader trait is new work with `crawler.rs` as its first
  implementer. `ux-events`, `document-host` and `gazette-ledger` have no dependents. The
  `https-blobs` and `iroh-blobs` features are never enabled. `mere-comms` is already out of
  the tree, pinned from murm.git (cycle-breaks D4 landed). `crates/crawl` and `crates/import`
  are referenced by nothing live. `quint-shaders` no longer exists (folded into
  `conatus::resident` on 2026-08-31); the spatial set is six crates.
- 2026-11-07: of the names P0 would free, `mere-cartography` has no bare form to reclaim:
  `cartography` on crates.io is a stranger's map library (0.12.0, 7.4k downloads). A proposed
  rename of Eidetic's `chartulary` to it was dropped on that finding (Mark: "Oh, meh then.").
  `stemma`, named with chartulary and scholia as the substrate's three, is also a stranger's.
- 2026-11-07: the 2026-06-18 benchmark measured a `Vec<Quad>` proxy at about 11x memory, 19x
  load and 18x mutation against petgraph truth with an identical hot path, because both used
  the same runtime index. The measurement, not the arena, carries forward.

## Progress

- 2026-11-07: plan opened; D1 to D5 recorded; nothing moved.
- 2026-11-07: D6 (seiche) and D7 (consolidations) recorded; phase C added.
- 2026-11-07: D8 (incipit) and D9 (catchment) recorded.
- 2026-11-07: D10 (squish, free names first) recorded; P0's default disposition is merge.
- 2026-11-07: Mark approved the plan, parallel lanes and non-Fable subagents. Opened: P1 (Inker)
  and P2 (Seiche) as worktree lanes on Opus, each stopping before any repository is created;
  P0's crate lines for the remaining crates as a read-only audit on Sonnet. Three tasks, the cap.
- 2026-11-07: P1 lane (worktree): P0 lines for the twelve Inker crates; Fleece's `crawl-graph`
  intake cut (generic session in Fleece, single-hop lowering to `mere-fetch`'s `fleece-crawl`);
  script-rhai's Scenograph test moved to `scenograph`; standalone Inker workspace checks and
  tests green with four fork-held edges (F1 to F4) and one file edge (F5); Mere, Knot and
  Turnstone check green. Reader styles (F1) and the `knot-editor-host` split (F2) wait on
  Mark. Nothing moved.
- 2026-11-07: P0 audit landed in the receipt file; its confirmations, corrections and forks are
  in P0 and Open. Eidetic's squish audit opened in the freed slot. P2's lane reported green
  (branch `worktree-agent-a3b1be946717468ea`, commit `07b40d612`); its plan lines are merged
  here and its forks are in Open.
- 2026-11-07: P1 lane reported: Fleece's `crawl-graph` cut to a generic `CrawlSession` with the
  HTTP lowering in `mere-fetch` behind `fleece-crawl`; `script-rhai`'s scenograph test moved
  to scenograph as a consumer test; the 12 Inker crates check and test standalone (761 pass,
  one 100 ms deadline test flaky under a concurrent build); Knot and Turnstone check against
  the post-move shape. Steps 3 and 4 came back as forks F1 and F2. Staged on
  `worktree-agent-a61f20e98db5486a2`, uncommitted behind the failing clippy hook. Turnstone's
  `--workspace` check pulled the CEF distribution into that worktree's `target/`, an
  unintended download to delete with the worktree.
- 2026-11-07: Eidetic squish audit landed in the receipt file; D11 to D14 recorded.
- 2026-11-07: the whole dirty tree committed on main as `0adc6b1dc` (D11); hooks passed, clippy
  skipped with no Rust staged. The two lanes' `target/` directories (4.1 GB and the CEF
  download) and the throwaway Turnstone and Knot check copies deleted; `seiche-ws` and
  `inker-p1-standalone` kept for the repository population. D15 to D18 recorded;
  the `gh repo create` calls for inker and seiche were refused by the harness and wait on Mark. Next lanes: Knot repin (D15),
  state-witness and mere-framing to Eidetic (D12), P1's F1 cut on its branch.
- 2026-11-07: D12 lane landed in Eidetic: branch `receive/state-witness-framing` at `58cfafe` in
  `Code/worktrees/eidetic-witness-framing`, copied from Mere `0adc6b1dc` with all 11 file hashes
  matching, manifests byte-identical, 18 lock lines added, check and clippy green, 37 tests
  pass; not merged, not pushed. Consumer steps are in Open. The wgpu-* squish audit opened in
  the freed slot.
- 2026-11-07: Mark created `merely-made/inker` and `merely-made/seiche`, empty, from the M4.
- 2026-11-07: the Eidetic lane had branched from a local main 64 commits behind origin; the push
  was refused. The branch was rebased onto origin/main (`71188bb`), the lock regenerated on
  origin's with the same 18 added lines, and the gates rerun before the push.
- 2026-11-07: Knot repin lane (D15) done on `repin/backdrops` at `2c703c4` in
  `Code/worktrees/knot-editor-repin-backdrops`, branched from origin's main (local main was 11
  behind): `sceno-backdrops` taken from graphshell.git per cycle-breaks D8, D9 and D17; five
  source files, four manifests, mere.git moved to `3b354929` and graphshell.git to `40e8c8df`
  in the lock, other families held at their old revisions; check, clippy and the composition,
  readings and desktop tests green; the Windows `composition_retention` failures are in Open.
  Not pushed. Mere relocks with `cargo update -p knot-editor` once it is on Knot's main
  (`support/family-promotion/source-Cargo.lock` holds six more lines if it follows).
- 2026-11-07: P1 lane under D16 and D17: `mere-lanes` owns `ReadingPreferences`, `ReadingAnchor`
  and reader styles (`reading_style.rs`, no serde; hosts persist and convert);
  `MarkdownLexer` and `full_pack` moved to `illume` behind a `markdown` feature with their
  tests; `knot-editor-host` re-exports them and says it is retiring; `tabard-workshop`
  converted. Standalone Inker: check green, 765 tests pass, 0 fail. Remaining edges:
  `knot-editor-host -> cambium` (until Knot takes the editor), `mere-lanes -> tabard` (F6,
  palette only), `mere-fleece -> mere-rdf` optional (F3), `script-rhai -> hest` (F4), the
  Roboto font (F5). 32 entries staged on `worktree-agent-a61f20e98db5486a2`, uncommitted
  until D15 lands and Mere relocks. The wgpu-* audit landed in the receipt file.
- 2026-11-07: Eidetic main pushed at `71188bb` (D12 landed); Knot main pushed at `2c703c4` (D15
  landed, fast-forward from origin's main); Mere main merged with origin's four viewer commits
  (no Rust on this side, nothing to gate) and pushed at `9e790a152`, so the plan is upstream.
  Mere's relock to Knot started. D19 and D20 recorded; a lane opened to move `tinct` into
  the Graphshell contract root.
- 2026-11-07: tinct lane landed: `merely-made/graphshell` cloned to `Code/repos/graphshell`;
  branch `receive/tinct` at `d80316db` in `Code/worktrees/graphshell-tinct`, an exact copy from
  Mere `9e790a152` (five of six blobs identical, the manifest inheriting workspace serde and
  repository), one lock entry added, workspace check and 308 tests green, README's History
  table updated; not pushed. Consumer row once on main: `tinct = { git =
  "https://github.com/merely-made/graphshell.git", branch = "main" }` (`tincture = { package =
  "tinct", ... }` for registry and pictograph). Mere then drops the member, the patch row and
  repoints two workspace rows; five outside repositories repoint their own row.
- 2026-11-07: Mere's relock to Knot `2c703c43`: `cargo update -p knot-editor` kept refreshing
  every other `branch = "main"` family source it touched (genet, eidetic, dramatis, gemot,
  bole, murm in turn), so the lock was restored and the eight Knot entries moved by hand, cargo
  then adding the one `sceno-backdrops` dependency line offline; `--locked` metadata is
  consistent and the clippy gate is running. D21 recorded, reverting D20 on the build-surface
  evidence.
- 2026-11-07: tinct pushed to graphshell main (`d80316db`). The relock committed on main. D22
  recorded; Stack 9 written with the eighth slot.
- 2026-11-07: D23 recorded. The Mere consumer-cuts lane (tinct, state-witness, framing out of
  Mere) first landed in a Graphshell worktree by a working-directory slip and did nothing;
  relaunched from Mere. P1 continuing on the palette over tinct.
- 2026-11-07: P2 lane (worktree, nothing moved): P0 lines for the spatial crates; the
  standalone workspace checks and tests green (372 passed, 11 ignored); the binding replay
  holds its recorded figures; canvas, Eponym and Mesocosm check against it. Stopped at the
  forks under P2, `state-witness` first.
- 2026-11-07: D12 and D19 donor side landed on a worktree branch: Mere deleted
  `state-witness`, `mere-framing` and `tinct` (each content-identical to Eidetic `71188bb` and
  Graphshell `d80316db`; tinct's manifest differs only in its comment and workspace serde) and
  their member, `framing` and patch rows; `pictograph` and `seiche` take the witness from
  eidetic.git, `tinct` and `tincture` come from graphshell.git. The lock was edited by hand:
  eidetic.git moved `6b4ec49` to `71188bb` (10 entries, plus the two moved crates) and
  graphshell.git `40e8c8d` to `d80316d` (10, plus tinct); upstream between those revisions only
  added the moved crates, nothing else moved, `--locked` metadata is consistent.
  `ports/graphshell/web/Cargo.lock` moved graphshell.git the same way (8, plus tinct);
  `support/family-promotion/source-*` are the 2026-10-19 checkpoint and stay. Workspace check,
  the pictograph, seiche, tabard and registry tests (460 passed, 12 ignored) and clippy green.
  Outside consumers are in Open.


- 2026-11-07: P1 lane rebased onto main `4b99ce9f2` (D15 landed). F6 settled by D19: the
  smolweb palette moved into `mere-lanes` over Graphshell's `tinct`, so the lanes no longer
  depend on Tabard; Mere's graphshell.git rows moved together to `d80316db`; F4 settled by D19
  (no change). Standalone Inker checks and tests green (769). Inker's edges into Mere are
  `knot-editor-host -> cambium` (until Knot takes the editor) and `mere-fleece -> mere-rdf`
  optional (F3), plus the Roboto font (F5). Mere checks green except the Graphshell port's
  five D16 sites, which hold the commit (the hook checks the whole workspace).
- 2026-11-07: P1 lane: the Graphshell port converts at its five D16 sites
  (`retained_content.rs`, two local anchor helpers and an inline preferences copy; no manifest
  change, persisted anchors still Tabard's); `cargo check -p graphshell --all-targets` green,
  the 19 reading and retained-content lib tests pass. Committed on the lane branch with the
  clippy hook run; not pushed.
- 2026-11-07: both code lanes merged into main through the hooks and pushed (`1ace7b211`): the
  consumer cuts (`209ebe6ff`, as `4bc6a2244`) and P1 (`9ae4df0f8`, as `c14a3c8ef`), each merge
  checked clean of conflicts except one Progress block kept from both sides. Three lane
  worktrees and their branches removed. Another session landed, in the same hour, a genet relock
  (`d634ee0c485`), a `seiche-model` split (the rapier-free model re-exported under the old paths)
  and the graphshell host plan's T7; the Seiche population must take `seiche-model` with the six
  crates. The Inker history lane is running (filter-repo on the twelve paths); its first push
  waits on Mark.
- 2026-11-07: Inker populated with history: `git clone --no-local` of Mere `f8c06097e`, filter-repo on
  the twelve crate paths and three design areas (477 commits kept, tree identical for 270
  files), root files added (workspace manifest from the proved standalone, lock reduced from
  1849 to 891 packages, toolchain, licences, README naming D21), the Roboto test font carried
  with its Apache-2.0 text (F5 closed for Inker); check green, 769 tests with P1's features,
  clippy warnings only; pushed to `merely-made/inker` main at `3109942` on Mark's word. Mark
  ruled the canonical DOC_POLICY core plus an Inker addendum and a DOC_README (a lane is
  writing them) and D24. The Mere donor lane for the twelve directories opened.
- 2026-11-07: P1 cut in Mere (worktree branch, not pushed). Deleted the twelve Inker crate
  directories and `inker_docs`, `nematic_docs`, `verso_docs` (blobs compared against
  `inker.git` `3109942`: identical but for the Roboto path in `document-canvas`'s text test);
  the root members, the twelve `[workspace.dependencies]` rows now `inker.git` with
  `branch = "main"`, nine `mere.git` patch rows dropped and five re-added as `inker.git`
  redirects for Knot. Lock edited by hand, one `cargo fetch` of `inker.git`: ten packages
  moved from path to `inker.git#3109942`, `graft-engine`, `weld-engine` and `welding` left
  (no consumer), no other source moved. The web workspace's two rows moved the same way
  (seven packages) with eight Cambium-family path redirects. The port's three portable-clip
  fixtures are copied to `ports/graphshell/tests/fixtures/portable_clip/` (Mark's ruling
  of 2026-11-07). `line-ceiling.toml` lost six Inker rows; `LICENSES.md`, DOC_README, DOC_POLICY
  and `moved_out.json` record the move; `support/family-promotion/source-*` is the
  2026-10-19 checkpoint and stays. The first check found a second file edge: `uxtree`'s
  probe test reads two Micron navigation probes from `nematic`; they are copied to
  `crates/forme/uxtree/tests/fixtures/micron/` under the same ruling. Gates (locked, offline,
  debug info off): workspace check green; `graphshell` lib 330 passed, 7 ignored, and the
  `retained_content_fixture` example builds; the `uxtree` probe test passes; clippy green
  (0 errors); the web workspace checks green.
- 2026-11-07: P1 complete on both sides. The donor cut (`bdd8f7ee7`) merged into main as `cd8490fa2`
  through the hooks and pushed: 289 files, 63,710 lines left Mere; Mere's rows name `inker.git`;
  five `mere.git` patch redirects stay until Knot repins; the port and uxtree carry their own
  fixtures (D24 applied twice). Inker main is `436c841` with its DOC_POLICY and DOC_README. The
  empty `crates/crawl` and `crates/import` directories removed. Opened next: Seiche population
  with history (seven crates, `seiche-model` included), Knot's repin to `inker.git` with the
  editor move (D17), Turnstone's repin with its D16 boundary conversions.
- 2026-11-07: Seiche populated with history at `Code/worktrees/seiche-history`: clone of Mere `4fa1686da`,
  filter-repo on the seven crate paths and the conatus README (159 commits kept, trees
  hash-identical, 197 files), root files as Inker's, no patch table needed (the `cubek-matmul`
  patch is Mere's own path and not carried), lock reduced 1847 to 859 packages; check, all-features
  check and clippy green, 372 tests pass (seiche-model's 9 split from seiche's 215), binding replay
  byte-identical to the recorded trace; main `a545239`, not yet pushed. Mark ruled: push after a
  second commit fixing the seven `repository` fields and the conatus README's links, and D25.
  Knot's lane committed `6f2ac20` (rows to inker.git, editor into `knot_document::retained`, 61
  and 78 knot-document tests, six Windows `cfg(not(unix))` failures apart); Mark ruled the lane
  repins Mere to current main so the lock holds one source per package, keeps the `retained`
  path, and the stale `.cargo/scope-navigation-local.toml` overlay in Knot's checkout is deleted.
- 2026-11-07: Seiche pushed to `merely-made/seiche` main `a6b8641` (161 commits; second commit set the
  seven `repository` fields, fixed the conatus README's links and founded `seiche_docs` with the
  conatus engine plan and the conatus/Nexus alignment brief under D25; the field system extraction
  note stays in Mere, it spans the kernel and the host). Mere's donor lane for the seven crates
  opened. Knot main `a58ba28`: Inker from `inker.git`, the editor as `knot_document::retained::KnotEditor`
  (D17), Mere repinned to current main with one source per package, the stale local overlay
  deleted. Turnstone main `841c8117`: nine rows to `inker.git`, D16 conversions in
  `src/shell/reading_edge.rs`, Knot's lock moved to `2c703c4`, five Inker and one tinct redirect
  rows in its mere.git patch table until Knot and Woodshed repin; one CEF download happened through
  the repository's own verify script and was deleted with the build directory. Both pushed on
  Mark's word; lane worktrees and branches removed. Free disk 481 GB after cleanup; Mark handles the
  rest himself. Next: Mere relocks to Knot `a58ba28` and drops its five redirects and the port's
  `knot-editor-host` use (after the Seiche donor merge); Turnstone's `knot_authoring` moves to
  `knot-document`; then `knot-editor-host` retires from Inker.
- 2026-11-07: P2 cut in Mere (worktree branch, not pushed). Deleted `crates/conatus/` (the seven
  crates and its README, trees as Seiche `a545239` took them) and D25's two documents; the
  member rows and the `seiche-model`, `nisus`, `modulus` and `conatus` workspace rows (no Mere
  consumer) are gone, `seiche` and `numen` name `seiche.git` on `branch = "main"`, and the two
  `mere.git` patch rows became `seiche.git` redirects for Knot and Cleromancy. The physics dev
  opt-level table stays: all six packages remain in both graphs. Lock edited by hand, one
  `cargo fetch` of `seiche.git`: `seiche`, `seiche-model`, `numen` and `conatus` moved from path
  to `seiche.git#a6b8641`; `nisus`, `modulus`, `tenant`, the kiss3d fork, `rapier3d` and 13
  registry packages only they reached left (1843 to 1823); no other source moved. The web
  workspace moved the same four and needed no patch rows (1139 packages either way).
  `LICENSES.md`, DOC_README, DOC_POLICY, `moved_out.json` and the seiche-replay README record
  the move; `line-ceiling.toml` named no moved file. Gates (locked, offline, debug info off):
  workspace check green; `pictograph` and `mere-kernel` tests 553 passed, 3 ignored, and
  `pictograph` with `canvas,gpu` 676 passed, 19 ignored; clippy 0 errors; the web workspace
  checks green. Outside repins are in Open (After P2's cut).
- 2026-11-07: P2 complete on both sides. The Seiche donor cut (`44f1dc0cb`) merged into main as
  `18e612dff` through the hooks and pushed: `crates/conatus` gone (197 files, 49k lines), `seiche` and
  `numen` from `seiche.git` `a6b8641` with two `mere.git` redirects for Knot and Cleromancy, twenty
  packages left the lock, pictograph and the kernel green (553 and 676 tests), the two D25 documents
  deleted with DOC_README pointers. An untracked `quint-shaders` remnant directory removed. Still
  owed by consumers: Isocosm (conatus, modulus, tenant, nisus rows and state-witness), Cleromancy,
  Knot, Murm's signalman-desktop and mer3ly repoint `seiche` rows to `seiche.git`; Isocosm's
  `.cargo/tabletop-local.toml` overlays numen and seiche onto deleted paths.
- 2026-11-07: Turnstone main `6c4d278b`: `KnotEditor` from `knot_document::retained` at Knot `a58ba28`,
  `knot-document` with its `engine` feature, the `knot-editor-host` row and the five Inker redirect
  rows gone (clean resolve, no unused patch); the `tinct` redirect stays while Woodshed names it
  through `mere.git`. Mere's matching cut opened (relock to Knot `a58ba28`, drop the five redirects,
  port's `block_editing` onto `knot-document`). Outside repins opened for Isocosm (conatus family
  to `seiche.git`, state-witness to `eidetic.git`, tinct to `graphshell.git`) and Cleromancy (seiche,
  tinct).


- 2026-11-07: Mere relocked to Knot `a58ba28` (worktree branch, not pushed). The eight `knot-editor.git`
  lock entries moved together from `2c703c4` by hand, one `cargo fetch`; no other source moved.
  Cargo printed no unused-patch warning for the five Inker redirects (their targets sit in the graph
  under their own source, so Cargo counts them used), but Knot `a58ba28` names none of the five
  through `mere.git`; removed, the graph resolves `--locked` unchanged. The Graphshell port takes
  `KnotEditor` from `knot_document::retained` (`block_editing.rs:14`; manifest 56 and 219 name
  `knot-document`), and the root `knot-editor-host` rows left with the package (1823 to 1822). The
  web workspace: `knot-document` replaces `knot-editor-host`, with a `mere-edit` path redirect
  beside its Cambium ones (1139 to 1141, `mere-edit` and `windows-sys` 0.59 added). Gates
  (locked, offline, debug info off): workspace check green; `graphshell` with `editable-blocks`
  green, `block_editing` 42 passed; clippy 0 errors; the web workspace checks green for wasm.
  `knot-editor-host` has no consumer left in Mere; after Turnstone's `knot_authoring` moves it
  retires from Inker.
- 2026-11-07: D17 complete everywhere. Mere's relock lane (`030633e0a`) merged as `a2c1cf8a5` and pushed
  with origin's catalog work as `462686e66`: Knot at `a58ba28`, the five Inker redirects gone, the port
  on `knot_document::retained`, the web workspace relocked with a `mere-edit` path redirect. Inker main
  `174ae7b` retires `knot-editor-host` and drops the unused Cambium row (627 tests, 0 failed); Inker's
  one remaining edge into Mere is Fleece's optional `mere-rdf` (F3). Two lessons for the receipts: the
  Seiche donor merge unlocked `sha2` to 0.11 beside `md-5` 0.10 and broke the full web build, which
  another session fixed in `eec6050c7` (the donor gates checked the web workspace, they did not build
  it); and the main checkout's shared `target/` was being rewritten by another session's cargo, so the
  merge gates now run in an isolated build directory with `CARGO_BUILD_JOBS=4`, after a fresh-directory
  check showed the earlier "can't find crate" errors were a parallel-build race, not code. Other
  sessions are already developing Seiche (`d6f44d8`, `seiche-model` catalog tables).
- 2026-11-07: outside repins landed on Mark's word. Isocosm main `753d9702`: twelve rows to `seiche.git`,
  `eidetic.git` and `graphshell.git`, nine locks moved (Mere to current main as well, since the old pins
  named the moved crates through `mere.git`), all nine workspaces green with all features, 1,036 tests,
  plus a one-line fix for the world-graph viewer's `prepare_presentation` hook that was already
  broken on main; its untracked `.cargo/tabletop-local.toml` overlay still names deleted Mere paths.
  Cleromancy main `e79576b`: `seiche` and `tinct` rows, two `seiche.git` redirects for Knot, Mere at
  current main, `with_resident_defaults` in the worker and one example (Mere `022560ff1` moved the
  divination registration; cycle-breaks P3 moves it into Cleromancy); core 60 tests green, root 147
  pass with two failures identical on unmodified main (an authored-layout aria state, a missing notes
  route fixture). Still owed: Knot's `seiche` row (then Mere's and Cleromancy's two redirects leave),
  Murm's signalman-desktop and mer3ly `seiche` rows, Woodshed's `tinct` rows (then Turnstone's tinct
  redirect leaves). Rust compiles hit out-of-memory three times while other sessions built; the
  machine is at its limit with every session running.
- 2026-11-07: P0 squish lane (branch `p0/squish`, worktree `worktrees/mere-p0-squish`, not merged).
  `mere-curation` merged into `incipit` as `incipit::curation` (585 lines and their tests moved, the
  crate's manifest deleted; five consumers repathed, pictograph by import paths only). incipit went to
  0.2.2, not 0.3.0: the external distillery repo takes incipit `^0.2` through the crates-io patch, and
  0.3.0 put a second incipit in the lock. `persona-picker` merged into `cambium` as
  `cambium::persona_picker` (256 lines and tests moved) behind a new optional `persona-picker` feature,
  so cambium's other consumers do not build the personae identity stack (about 160 crates); no cycle,
  since neither dramatis nor personae reaches cambium. Names freed: `mere-curation` (on crates.io only
  once its 0.1.1 publish is yanked or left) and `persona-picker`. D14 is held at a fork: `mod_loader`
  emits through `diagnostics` (`emit_event`, `DiagnosticEvent`, eight `CHANNEL_MOD_*` constants; its
  tests use `diagnostic_channel`, `scoped_thread_sender`, `IngressLimits`), diagnostics is 4,346 lines
  and not 956, and Turnstone names `mere-registry` with only the `diagnostics` feature, so moving
  diagnostics into the resident would make document-host and Turnstone depend on a port.
  `mere-registry` is not yet freed. Before the branch reaches main: Knot, Turnstone and Woodshed name
  `mere-curation`, and Woodshed and Dramatis name `persona-picker`, from `mere.git` main.
- 2026-11-08: P0 registries lane (branch `p0/registries`, worktree `worktrees/mere-p0-registries`, not
  merged). D14 carried out in its amended form. Deleted from `mere-registry`: `input`, `knowledge`,
  `layout`, `lens` and `protocol` with their features (4,192 lines of Rust plus the 162-line UDC seed
  under `assets/knowledge`); default features are now `["diagnostics"]`; manifest dependencies dropped:
  hest, http, inker, inventory, kernel, nucleo, parking_lot, petgraph, serde, serde_json, tabard,
  tincture, toml, tower, and the dev-dependencies serde_json and tempfile. `mod_loader` (2,385 lines,
  tests included) moved into `document-host` as `document_host::mod_loader`, its only consumer; the
  `mod-loader` and `test-utils` features are gone (no consumer in the tree; the test-only helpers are
  now `cfg(test)`). document-host takes `registry` with `diagnostics` because the loader emits through
  it. `mere-registry` is kept under its name as the diagnostics and observations crate rather than
  freed: moving diagnostics into the resident would make document-host and Turnstone depend on a port,
  and Turnstone and the resident name the crate today, so trimming in place costs them nothing.

- 2026-11-08: P2's consumers repinned and pushed: Murm `df3e293`, mer3ly `47599d6`, Knot `189487c1`
  and Woodshed `648ae565`. Each takes `seiche` from `seiche.git`, `tinct` from `graphshell.git`,
  curation as `incipit::curation` and the persona picker as cambium's `persona-picker` feature, with
  a Mere relock to `1aa838c2` wherever its lock pinned a pre-split Mere. Mere relocked Knot from
  `a58ba28` to `189487c1` (eight root entries and one web entry, by hand, one `cargo fetch`, no other
  source moved) and dropped the `numen` redirect row; the lock holds one `seiche`, one `numen`, one
  `incipit` and no `mere-curation`. The `seiche` row stays: `mere-resident` locks Cleromancy
  `671e9bc4`, whose `cleromancy-core` names `seiche` through `mere.git` (Cleromancy repinned it in
  `e79576b`). Later the same day: Cleromancy `d9fc819` (Mere relocked to `52047e58`, Knot to
  `189487c1`, both redirect rows gone), Turnstone `f3a455d3`, `c24b5e0d`, `b12348d9` (Mere, Knot and
  Woodshed relocked, curation repathed, its `seiche`, `numen` and `tinct` redirects gone) and
  Dramatis `772cce49` (app workspace relocked 154 Mere commits, the picker as cambium's feature)
  pushed; Mere then relocked Cleromancy to `d9fc819` and dropped its last redirect row, `seiche`.
  No family consumer names a moved crate through `mere.git` any more, and no `[patch]` redirect
  for one remains in Mere, Turnstone or Cleromancy. Pre-existing failures unchanged: Knot's three
  `site_generation` tests (same at `a58ba28`), Turnstone's 46 "Access is denied", Cleromancy's two.

- 2026-11-08: D27 carried out (branch `p0/backstep`, worktree `worktrees/mere-backstep`, not merged).
  `backstep` 0.2.0 (0.1.0 is the published version; this one is not), still dependency-free, now
  holds three jobs: snapshot undo, a text `Clipboard` trait (`read_text`, `write_text`, errors
  `Unavailable`, `Refused`, `Empty`; `NoClipboard` refuses) and `backstep::nav::History`, chrome's
  linear back/forward (71 lines and its five tests moved; `NavTarget`, `classify`, `resolve_href`
  and the query encoding stay in chrome, which takes `backstep` and re-exports the type as
  `chrome::nav::History`, so Turnstone's uses still resolve; the lock gains only that edge).
  Bound: Cambium's `TextInput::apply_clipboard` with Ctrl/Cmd+C/X/V as `ClipboardCommand`. Left
  as forks, the shape being no synchronous call: rootstock's egui embedding (Copy and Cut need
  the document's selected text, which Genet's `document-session-api` does not expose; Paste
  already arrives as text), rootstock's iced embedding (iced's clipboard is request-shaped:
  `Event::Clipboard` carries `Read` and `Written` results of earlier requests), chrome's frame
  model (writes travel back through host ports) and the Graphshell web share (a browser promise
  inside a user gesture). Knot's `backstep = "0.1.0"` requirement needs 0.2.0 when it next repins.
