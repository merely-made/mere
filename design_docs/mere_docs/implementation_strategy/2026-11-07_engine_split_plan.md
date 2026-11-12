# Engine split plan: Mere as the semantic-web engine, Graphshell as the host, Inker and Conatus as suppliers

**Status:** 2026-11-08: D1 to D30 are ruled (D21 reverts D20; D30 supersedes D29's frame-model shape). Moved:
`merely-made/inker` holds eleven crates with history (the twelfth, `knot-editor-host`, retired under D17; main
`174ae7b` at 2026-11-08) and `merely-made/seiche` holds the seven spatial crates, `seiche-model` included (main
`4109f0b` at 2026-11-08); Mere takes both as `branch = "main"` git dependencies and every family consumer
named in Open is repinned (P1 and P2, below). P0's squishes have landed in part (D14, D26 to D28). P3 (the
hosting in Graphshell), P4 to P8 and the consolidations C have not started: `errand`, `emblem` and `wavicle`
are still separate repositories. The engine-to-workspace contract is deferred by D4; the Pandect sidecar
owners and the lanes' forks are open.

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
the resident's own split (resident split plan (`graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-05_resident_split_plan.md`)); domain
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
  Graphshell. D7 meant to pay for them with three folds, but only wavicle still folds: D32
  keeps errand and iconography D1 keeps emblem. So the family nets 26 repositories from 25.
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
persona and domain catalog, access, store lifetimes, sealing, replication), `mere-facts` (the persona's facts domain; its admission policy stays with
the domain owner, as the contract says). The reservoir service crate is `catchment` (D9). `mere-framing` and `state-witness`
moved to Eidetic under D12 and are no longer Mere's.

**Graphshell: the host.** Workspace half of `graph-kernel` as the workspace kernel, which
grows out of `incipit` and takes its name (D8: surfaces, trails, arrangements, bindings to
resource references by UUIDv5; the graph and session ids stay reachable behind a minimal
default feature so Cleromancy and Fleece keep depending on ids alone),
the `mere` facade's navigation, trail and roster logic (its visual and workbench APIs to the
view and composition crates; no new facade), `mere-cartography` (not `pictograph`, which goes to the emblem repository under iconography D1), `mere-forme`
(merged into `incipit` as `incipit::forme`, P0 done),
`mere-curation` (merged into `incipit`, P0 done), `platen`, `uxtree`, the Cambium family (`meristem`, `cambium`,
`cambium-rootstock`, `cambium-winit`, `cambium-a11y`, `cambium-desktop`, `cambium-web`,
`workbench`, `hest`, `backstep` (kept and grown under D27 into the editor's ephemeral state: snapshot undo, the
host clipboard contract, chrome's linear back/forward; P0 lane `p0/backstep`, not merged), `mandrake`, `sprigging`, `tinct`, `persona-picker` (merged into `cambium`, P0 done),
`mere-view`, `mere-overmap`), `tabard`, `mere-chrome` (dissolved under D31, P0 done: `routing` into `ux-events`,
`nav::History` is backstep's, the rest retired; `mere-chrome` freed), `mere-registry` (examined feature by
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
`scenotime`, the WIT world) are the same repository's bottom layer. Several crates named above are
already in that repository (2026-11-08): `chirograph`, `graphshell-client`, `graphshell-endpoint`, `hest`,
`sceno`, `scenotime`, `script-wire`, `script-witgen`, `tinct` (D19), and, from cycle-breaks D17 and P7,
`sceno-backdrops` and `vector-sheet`; the hosting list above names what still moves there from Mere.

**Inker.** `inker`, `document-canvas`, `graft-engine`, `scrying-engine`, `weld-engine`,
`nematic`, `illume`, `mere-lanes` (taking reader styles and owning its reading-anchor type; the host
adapts Tabard at the edge), `mere-pageload`, `mere-fleece` (generic traversal, clipping and
capture; its `crawl-graph` intake adapter goes to the host), `script-rhai` (eleven crates; `knot-editor-host` retired under D17, its lexers in `illume` and its editor in Knot's
`knot-document`). Inker emits a
document, a predicate-bearing link or a JSON-LD block with its base address; the engine
consumes RDF; the host decides what to keep.

**Seiche** (the spatial repository, D6). `conatus`, `modulus`, `nisus`, `tenant`, `numen`, `seiche`
and `seiche-model` (`quint-shaders` no longer exists; it folded into `conatus::resident`). Isocosm's Eponym
and Mesocosm already consume the family without the graph.

**Elsewhere.** `mere-alembic` to Distillery (it says it is Distillery's component);
`mere-comms` is in Murm (cycle-breaks D4, landed); `chirograph` is in Graphshell (D9, landed). The empty
directories `crates/crawl` and `crates/import` are gone. Forks in `Code/crates`: `boa`, `bole`,
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
  *Update 2026-11-08: done.* Knot `a58ba28` and Turnstone take every Inker crate from `inker.git`; Mere's
  `mere.git` patch table holds no Inker or Seiche redirect row; Inker (`174ae7b`) no longer names Cambium (it
  dropped the unused workspace row when `knot-editor-host` retired), so its one remaining Mere edge is the
  optional `mere-fleece -> linked-data` row (F3). `knot-editor-host` has no consumer in any manifest.

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
`KnotEditor`, `EditOutcome` and `KnotReadout` from `crates/inker/knot-editor-host/src/model.rs` *(historical citation)* <!-- doc-audit: historical-path --> and `crates/inker/knot-editor-host/src/editor.rs` *(historical citation)* <!-- doc-audit: historical-path -->
(with `model.rs`'s three tests) and depends on `cambium` and `illume` (feature `markdown`)
instead of `knot-editor-host` (root `Cargo.toml` line 106, `crates/knot-document/Cargo.toml`
line 38); Turnstone's `src/knot_authoring.rs:43` and Mere's `ports/graphshell/src/block_editing.rs:14`
then import `KnotEditor` from `knot-document`, and their manifests drop `knot-editor-host`
(Turnstone `Cargo.toml` line 268). Then `knot-editor-host` retires and leaves Inker.

- F2 (ruled D17, Knot's `knot-document`), `knot-editor-host -> cambium`: stays until Knot
  takes the editor (above).
- F3 (ruled D34: retire), `mere-fleece -> mere-rdf` (optional, `structured-data`, no consumer in any manifest).
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
  `crates/conatus/` *(historical citation)* <!-- doc-audit: historical-path -->, takes `seiche` and `numen` from `seiche.git` (`pictograph` and
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
- Needs: P1 (Inker as a supplier), P2 (Seiche as a supplier), cycle-breaks P6; the checker
  change of D35; past the first slice, the dissolution of the app's linked modules (D67, which replaced D41).

**Drift line for P3 (2026-11-08, from the D29 lane; Mark: "Record it as a P3 drift line; keep the
contract").** `mere-chrome`'s frame model (`FrameHostInput`, `FrameViewModel`, now `FrameStep` with
hest `Invocation`s out and `Outcome`s in, and `chrome::effects`) has no consumer: nothing in Mere,
Graphshell or Turnstone constructs the view model or consumes the host input, and `GraphshellRuntime`
exists only in doc comments. Superseded the same day by D30: the frame model retires and chrome is
audited for dissolution; D29's shape is the rule for rootstock's embedding and any future runtime step.

**Slice 2: dissolve the app's linked modules (D67 to D73, D81 to D83).** The reference app
(`ports/graphshell`) cannot leave for Turnstone while anything below Turnstone links it. Four
places do: `mere-resident` (13 modules, about 23k lines, 36 non-test use lines), Cleromancy
(`personal_sync`), Woodshed (a dev row for one conformance test) and Knot (the optional
`block-editor` feature). The resident serves these modules and never calls the app, so each
module moves to its owner below both sides. No route is added. Steps, in dependency order:

- **S1, leaves with no dependents among the others.** `declaration` to `graphshell-door`;
  `kept_text` to Eidetic (D71); the goal surface's provider, facts and door to
  `graphshell-host` (D91); the whole of `coop` to `graphshell-host` (D88), after `declaration`; `carriage` and
  `receipts` to the resident's `sync/` (D81), with the `receipt_ingest` bin (D92).
  - Done when: the resident and the app compile without those `graphshell::` paths, and the
    moved tests pass with unchanged counts.
- **S2, the sync domain.** `access` and `personal_sync` to pandect (D69), which gains
  stickleback and p2panda.
  - Done when: Cleromancy's `graphshell` row is gone. The package still reaches it through
    `mere-resident` until S6, which carries the no-package condition.
  - Needs: P4's pandect stages landed, since both edit pandect.
- **S3, the endpoint closure.** `mere_host`, `handlers`, `session_item` and `resident_host`
  become graphshell-host's `mere-host` feature (D68). The app's product and capture methods on
  `MereHost` become extension traits (D93), and the workshop methods stay in the app (D94).
  - Done when: the resident's route provider and the app's roughly 60 users build against it.
- **S4, transfer.** The data types, `transfer_offer` and the codicil decode move down with the
  sync domain (D70); the resident's tests stop calling `product`.
- **S5, the outside consumers.** Woodshed's conformance test runs over `block-host` (D72);
  `BlockEditor` moves to cambium without its widget accessors, and Knot implements it there
  (D73); the resident's MCP test moves to Turnstone with `graphshell-mcp` (D82).
- **S6, close.** The resident's four rows naming the app or `graphshell-mcp` are deleted, and
  consumers are re-addressed (D66).
  - Done when: `cargo tree -i graphshell` is empty for mere-resident, Cleromancy, Woodshed and
    Knot, and the family cycle checker passes with the app placed in Turnstone.

Another session is active in `ports/mere-resident`; each step rebases before it lands and
touches the resident last.

**Slice 2 landed (2026-11-10).** `mere-resident` names the reference app nowhere (Mere
`cc5d2161d`), and `cargo tree -i graphshell` finds no package in the resident, Cleromancy
(`704ac93`), Woodshed (`de16948e`), Knot (`13a729b`) or Turnstone (`ce581fd5`). The family
cycle checker exits 0 over 35 repositories and 92 lock roots. Where each module went:
`declaration` to `graphshell-door`; `carriage`, `receipts` and `receipt_ingest` to the
resident's `sync/` (D81, D92); `kept_text` to Eidetic; `access`, `personal_sync`,
`transfer_offer`, the transfer records and the codicil decode to pandect (D69, D70); `coop`,
the goal surface's provider, facts and door, and the mere endpoint closure to `graphshell-host`
features (D88, D91, D68); `BlockEditor` to cambium (D73, D90); the block adapter contract to
`block-adapter` (D95 to D97); the resident's MCP tests to Turnstone (D82). The app keeps
`ProductHost`, `CaptureHost` and `WorkshopHost` as extension traits over `MereHost` (D93, D94).
Left open: `graphshell-host`'s `mere-host` feature names the `mere` facade once, for
`mere::coupling_edit`, which slice 3 moves with the facade (D58); `mere-host` costs 462 crates
because `MereHost` uses pictograph's canvas; Cleromancy's `inference_sync` and `situation_sync`
tests call a now-private `Graph::facets_mut()` and did not compile before this slice either;
no moved code was checked headed, and only the moved tests were run.

**Slice 3: dissolve the `mere` facade (D58, D98 to D101).** Measured 2026-11-10 on `b7f05765a`:
`crates/mere` is 3,522 lines in eight modules plus six re-exports. Family code names
`mere::kernel` 447 times, `mere::canvas` 393, `mere::navigation` 18, `mere::platen` 15,
`mere::forme` 11, and the rest four times or fewer; `mere::linked_data` and `mere::workbench`
are named nowhere. Steps, in dependency order:

- **T1, the modules.** `navigation`, `trail` and `roster` to incipit (D58); `gloss` and
  `glossary` to mere-view's `gloss` feature (D98); `coupling_edit` to graphshell-host's
  `mere-host` (D99); `clip` to a graphshell-host `clip` feature (D100); `routing` to Turnstone
  (D101), which lands before Mere deletes it.
  - Done when: the facade holds re-exports only, and the moved tests pass with unchanged counts.
- **T2, the re-exports inside Mere.** Every Mere crate names `kernel`, `pictograph::canvas`,
  `incipit::forme`, `platen` and the rest directly, and takes over the pictograph features the
  facade forwarded (`physics-engine`, `image-decode`, `canvas-gpu`, `canvas-meaning-gpu`).
  - Done when: no crate in Mere depends on the `mere` package.
- **T3, the outside consumers.** Turnstone, Cleromancy, Isocosm, Murm's comms and any other
  family manifest naming the package repoint the same way and drop the row.
- **T4, close.** `crates/mere` is deleted and consumers are re-addressed (D66).
  - Done when: `cargo tree -i mere` finds no package in any family lock root, and the family
    cycle checker passes. The name is then free for P5's engine crate.

**Slice 3 landed (2026-11-10).** `crates/mere` is deleted (Mere `ec7ba066e`). No family manifest
names the package, and `cargo tree -i mere` finds none in Mere, its web root, Turnstone
(`fd57dd3f`), Cleromancy's three roots (`2d02f65`) or Knot (`e0b3374`); no other family lock
carried one. The cycle checker exits 0 over 35 repositories and 92 lock roots. Isocosm and
Murm never named the facade. Repoints: 438 in Mere, 369 in Turnstone, 100 in Cleromancy.
Feature sets were compared before and after and are identical in Mere; outside it, Turnstone
drops the unused `linked-data` default, Cleromancy's `core` and `authority-guest` drop the
unused `incipit/arrangement`, and `graphshell-blocks` keeps genet-parley's `complex-scripts`
through a direct row. Not verified: anything headed, Turnstone's non-default feature builds,
and full test suites beyond the app's lib tests and the moved tests.

**Slice 4: the toolkit and the app leave (D102 to D107).** Steps, in dependency order:

- **U1, the canvas fold (D103, D38).** In Mere, `pictograph`'s `canvas` and `signals` move into
  `mere-cartography`; `pictograph` keeps the faces. Every user repoints.
  - Done when: `pictograph` has no `canvas` or `signals` feature, the test attributes (820
    measured across both crates on `0c4f551d6`, not D38's 777) are
    accounted for, and the feature sets of the app, the web crate and the resident are unchanged.
- **U2, the kernel's hest edge (D105).** `kernel::actions` moves to `hest` in Graphshell's
  contract root; the kernel drops the row.
  - Done when: `mere-kernel` names nothing from `graphshell.git`.
- **U3, Graphshell's docs (D107).** `design_docs/` and the canonical `DOC_POLICY.md` core in
  Graphshell; the list of host-layer documents is ruled, then moved.
- **U4, the cutover (D102, D104, D106).** With history: the app, `graphshell-blocks`,
  `graphshell-mcp`, `graphshell-mere` and `graphshell-dramatis-pane` to Turnstone; `athanor` to
  the Distillery repository (D114); the other 32
  toolkit crates, `graphshell-door` and `block-adapter` to Graphshell's `host/`; the faces-only
  `pictograph` to the emblem repository. Mere keeps `mere-kernel`, `mere-rdf`, `catchment`,
  `pandect` and `mere-facts`.
  - Done when: Graphshell builds the host and the `mere-resident` assembly with Mere as a git
    dependency; Turnstone builds the app; Mere's workspace holds the five crates and builds.
  - Landed 2026-11-11, pending consumers (U5): the receivers' branches and Mere's deletion
    (Progress, 2026-11-11). Turnstone receives the app in its U5 step (D121).
- **U5, the consumers.** In strict order with Turnstone last (D121); Inker needed no change (D124).
  The 151 rows in Cleromancy, Isocosm, Knot, mer3ly, Murm, Woodshed and
  Turnstone are re-addressed to `graphshell.git` (D66) and their 20 lock roots relocked.
  - Landed 2026-11-11 (Progress, 2026-11-11): six consumers relocked, then Turnstone with the app.
- **U6, close.** Mere's residue goes: patch rows, line-ceiling exceptions (127 of 150 name
  leaving paths), scripts and hooks that name moved crates.
  - Done when: the family cycle checker passes, no Mere crate depends on `graphshell.git`, and
    P3's done-conditions are checked one by one and recorded.

Held, not part of this slice: D52 (the three small hosts merge) and D64.

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
- The dataset behind the query stage is designed and ruled in the [indexed dataset brief](../technical_architecture/2026-11-09_indexed_dataset_brief.md) (D1 to D5, 2026-11-09): the generic id-level index in chartulary, the `spareval` seam here, dense ids rebuilt per revision, six orderings, one index for live and history. Its §1 measured the borrowed adapter, which is what this phase replaces.

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

One repository folds into its consumer (not yet, 2026-11-09), as a workspace member of the receiver
with history preserved as the 2026-10-19 promotion did. `merely-made/errand` (the smolweb protocol family, misfin
included) stays its own repository under D32 and D33. `emblem` (IconVG) no longer folds: iconography D1 (2026-11-09) keeps it as its own
repository and moves `pictograph` (faces only, after D13 and D38) into it, because faces gained consumers below Graphshell. `wavicle` (WavPack) into
Woodshed, its only consumer, on Woodshed's own schedule. The four `wgpu-*` repositories
stay (D7, D21), with Inker recorded as the owner of scrying and welding. Inside each wgpu
repository the audit's folds apply: the `grafting-servo` rename finished,
the scry and weld demo crates folded, the three `keyutils.rs` copies deduplicated.
- Done when: each folded repository is archived with a pointer to its new home; its crates
  publish from the receiver under their existing names; every consumer resolves them from
  one source; the family's repository count is recorded.
- Needs: nothing in this plan for wavicle. Errand stays under D32; emblem left this section under iconography D1.

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

- **D29. A host-enacted effect from chrome is a hest Invocation; no clipboard enum on the frame
  model (Mark, 2026-11-08):** D27's F2 lane bound rootstock's iced embedding and the Graphshell web
  share to backstep's request/completion pair and found the frame model's outgoing leg had nothing
  to ride on: its "host ports" are a 2026-06 doc comment with no type in Mere, Graphshell or
  Turnstone. Offered a drained request list, a new port type or leaving it: "Are enums really the
  best approach here? Isn't this plan a bit stale relative to newer work on the subject?" Evidence:
  hest's `Invocation` (id, schema, opaque arguments, `EffectClass::ExternalEffect`) and host-established
  `Outcome` are the family's vocabulary for a runtime asking its host to enact an effect, mirrored on
  the wire by chirograph's `AdvertisedAction` with `IntentEffect::ExternalEffect` and `IntentResult`,
  already used by the Graphshell port's handlers and transfer endpoint and the resident's identity
  endpoint; chrome does not speak hest yet. Put back as hest on the frame model with Outcomes back,
  hest at the frame model with backstep's pair kept at the bindings, retiring the pair for hest, or
  leaving it: "A hest Invocation with ExternalEffect; Outcome back (Recommended)". chrome takes hest;
  the frame model's `clipboard_completions` becomes hest Outcomes keyed by invocation id and carries
  no clipboard-specific type; the stale comment is rewritten; the host enacts the write through
  backstep's binding or its own clipboard; the same shape later carries webview creation and focus
  handoff. backstep 0.2.0 (published 2026-11-08) keeps the pair for host-internal bindings.
  Outcome: refines D27 (the pair is a binding shape, not the runtime-to-host contract) and applies
  Stack 9 (chrome names the contract root's vocabulary rather than minting its own).

  Links: area The stack · draws on engine-split D27, engine-split D22 · plans engine-split P3 · summary chrome asks its host to enact an effect with a hest Invocation and reads a hest Outcome; the frame model carries no clipboard-specific type.

- **D30. The frame model retires; chrome is audited for dissolution (Mark, 2026-11-08):** the D29
  lane found that nothing in Mere, Graphshell or Turnstone constructs `FrameViewModel` or consumes
  `FrameHostInput`. First ruled "Record it as a P3 drift line; keep the contract"; then, asked what
  to do with the contract: "I'm alright with retirement but want to know if we would need the
  contract and the runtime in the future. What was going to depend on it? Does something need to
  become that shape?" The answer, from the tree: the pre-pivot desktop shell depended on it (the types
  lived in `shell/desktop/ui/frame_model.rs` in April; meerkat's egui shell painted a view model each
  frame and fed the host input back; they moved to graphshell-core on 2026-06-07, two days before the
  2026-06-09 pivot to Cambium over Genet retired that shell, and no consumer followed). The shape is
  live twice already, built where its consumer is: chirograph's projection sessions across a process
  or wire, and cambium-rootstock's embedding (per-toolkit `Input` in, `OwnerInput` out, the clipboard
  as request and completion) inside a foreign immediate-mode host. A future embedding takes
  rootstock's shape, and D29's hest Invocation and Outcome apply there. By module path the only chrome
  code any crate imports is `routing::ToolSurfaceReturnTarget` and `toolbar` (ux-events) and
  `nav::History` (Turnstone; now backstep's); omnibar, suggest, authorities and host_intent (1,270
  lines) have no importer and Turnstone carries its own omnibar view. "Retire the frame model now;
  audit the rest of chrome for dissolution (Recommended)": `frame_model`, `effects` and `host_intent`
  are deleted (git keeps them); a read-only audit confirms the other modules' consumers, after which
  chrome dissolves under D10 (routing and toolbar to their users; the name freed). Outcome: refines
  D29 (its shape is the rule for rootstock's embedding and any future runtime step, not for a kept
  type) and replaces the P3 drift line above; applies D10.

  Links: area The stack · draws on engine-split D29, engine-split D10 · plans engine-split P0 · summary The frame model retires with the shell that needed it; rootstock's embedding and chirograph are the living shapes; chrome is audited for dissolution.

- **D31. chrome dissolves (Mark, 2026-11-08):** the D30 audit (receipt file, 2026-11-08) found no
  chrome module with two consumers: `authorities` (400 lines), `omnibar` (398), `suggest` (326) and
  `toolbar` (215) have no importer and Turnstone and Pelt carry their own; `routing` (69) serves one
  type, `ToolSurfaceReturnTarget`, to ux-events alone; `nav` (259) serves Turnstone four uses of
  `History`, which is backstep's re-exported. The crate is 0.0.1 and unpublished. "Yes, dissolve
  chrome": the four modules and nav's classifier retire (1,598 lines; git keeps them), `routing`
  goes "Into ux-events, its only consumer", Turnstone swaps its `mere-chrome` row and four paths
  for `backstep`, the member and workspace row go, and `mere-chrome` is freed. Doc comments in the
  kernel and ux-events that name the retired types are cleaned in the same lane. Outcome: applies
  D10 and D30; chrome joins the names freed in P0.

  Links: area The stack · draws on engine-split D10, engine-split D30 · plans engine-split P0 · summary chrome dissolves: routing to ux-events, nav::History is backstep's, the rest retires; mere-chrome is freed.

- **D32. Errand stays its own repository (Mark, 2026-11-09):** Mark asked: "Should errand go
  to inker? Or does it contain unique transports that would put it in murm?" The evidence lane
  (receipt file, 2026-11-09) answered neither. Errand is 14 crates and about 33.9k lines, and
  nine crates in six repositories consume it: Mere, Inker (`nematic` for parsing only,
  `mere-lanes` for transport), Turnstone, Woodshed, Gemot and Murm. The `inker` crate does not
  consume it. No errand crate carries a peer carrier: those are all Murm's, and errand speaks
  client/server TLS, TCP and UDP to the public small web. Its 2026-08-03 home decision gives
  the protocol crates to their communities. Ruling: "Stays its own repo (Recommended)". D7's
  smolweb clause is withdrawn. Later the same day, iconography D1 kept emblem too, so C folds
  only wavicle and the family nets 26. Outcome: refines D7, which rested on a consumer premise the manifests do not bear
  out.

  Links: area The stack · amends engine-split D7 · plans engine-split C · summary merely-made/errand stays a supplier repository below Inker and the host; D7's smolweb fold is withdrawn.

- **D33. Misfin stays with errand (Mark, 2026-11-09):** misfin is 5.4k lines, a TLS mail
  protocol rather than a carrier. Its only real consumer is Murm's `mere-comms` mailbox adapter
  (247 lines), and its crates.io name is held in stewardship for the protocol's author. Ruling:
  "Stays with errand (Recommended)". Murm keeps the adapter, and errand does not come to name
  Murm. Outcome: settles the open question "Misfin's home"; applies D32.

  Links: area The stack · draws on engine-split D32 · plans engine-split C · summary misfin stays in the errand repository; Murm keeps its comms adapter.

- **D34. Fleece's structured-data feature retires (Mark, 2026-11-09):** P1 F3. The feature
  (561 lines in `structured_data.rs`, enabled by no manifest) is Inker's last edge into Mere. It
  keeps Inker's 11-row mere.git patch table and a mere→inker→mere repository loop, and it
  returns `GraphContribution`, which P5 sends to the host's legacy adapter. Ruling: "Retire it
  (Recommended)". The feature and its `linked-data` row are deleted (git keeps them), and a
  host that needs JSON-LD blocks composes Inker's block with the engine's parse. Outcome:
  settles P1 F3 and meets P1's done-condition once Inker lands it.

  Links: area The stack · draws on engine-split D1 · plans engine-split P1 · summary mere-fleece's structured-data feature and its mere-rdf edge retire; Inker names nothing in Mere.

- **D35. The family cycle checker treats each workspace root as a node (Mark, 2026-11-09):**
  the P3 cut map (receipt file, 2026-11-09) found that `check_family_cycles.py` blocks every
  P3 slice. It reads cycles at repository level, so the first Graphshell host crate that names
  mere.git closes graphshell↔mere (Mere's engine names hest and sceno from the contract root),
  and graphshell↔inker likewise. Ruling: "Each root is a node (Recommended)". The checker assigns
  manifest rows to a workspace root by path prefix and checks the root graph. This is D19's
  lockfile order made mechanical, and the allowlist stays for edges ruled but not yet broken.
  Outcome: refines cycle-breaks D6 (the unit is a workspace root, not a repository); applies
  D2 and D19.

  Links: area The stack · amends cycle-breaks D6 · draws on engine-split D2, engine-split D19 · plans engine-split P3 · summary The cycle checker's node is a workspace root; Graphshell's contract root and host workspace are separate nodes.

- **D36. Pelt's and Tabard's binaries go to Turnstone; their libraries go to Graphshell (Mark,
  2026-11-09):** the dispositions list put `pelt`, `pelt-desktop` and `tabard-desktop` in
  Graphshell, while D23 says Graphshell holds no application. Ruling: "Only binaries to
  Turnstone". The `pelt` and `tabard-desktop` binaries move to Turnstone. `pelt-core`,
  `pelt-desktop` and `tabard-workshop` move to Graphshell's host workspace as libraries.
  Outcome: applies D23.

  Links: area The stack · draws on engine-split D23 · plans engine-split P3 · summary Applications go to Turnstone and their libraries to Graphshell; pelt and tabard-desktop are binaries in Turnstone.

- **D37. Graphshell's host workspace sits at host/ beside the contract root (Mark,
  2026-11-09):** Ruling: "host/ beside the root (Recommended)". The repository root stays the
  contract workspace with its own lock and `exclude = ["host"]`. `host/Cargo.toml` is the
  hosting workspace with its own lock. It mirrors Mere's subpaths (`host/crates/cambium/…`,
  `host/ports/…`), so history imports and line-ceiling prefixes carry over, and it names the
  contract crates by path. Outcome: applies D2, D19 and D35.

  Links: area The stack · draws on engine-split D2, engine-split D35 · plans engine-split P3 · summary Graphshell's hosting workspace is host/ with its own lock, beside the unchanged contract root.

- **D38. Signals moves with the canvas; pictograph is faces only (Mark, 2026-11-09):** D13's
  "4.4k lines its name describes" counted `signals` (2,828 lines) as faces, but faces are 1,294
  lines. The canvas uses `crate::signals` 58 times in 25 files, and signals uses
  `cartography::IntelligenceSignals`, so leaving signals behind would make
  cartography→pictograph→cartography a cycle. Ruling: "Signals moves with canvas
  (Recommended)". `mere-cartography` becomes about 71.7k lines: the strategies, the canvas and
  signals. `pictograph` is about 1.3k, and it goes to the emblem repository under iconography
  D1. The 777 test attributes carry over. Outcome: refines D13's
  figure; its intent stands.

  Links: area The stack · amends engine-split D13 · plans engine-split P3 · summary mere-cartography takes the canvas and signals; pictograph keeps node faces only.

- **D39. A crate with no dependents is kept unless it declares itself superseded (Mark,
  2026-11-09):** P0 F8, F13 and F12's `document-host`. Three crates have no dependent manifest
  in ten repositories: `ux-events` (2,183 lines, routing's home under D31), `document-host`
  (3,722, the mod loader under D26) and `gazette-ledger` (477, published 0.0.1, whose own
  comment says it retires at the Dramatis split, which landed 2026-11-03). Ruling: "Keep unless
  self-declared (Recommended)". `gazette-ledger` retires, after its parity test is checked
  against the shared derivation. `ux-events` and `document-host` are kept as host leaves and
  move with P3. Outcome: new; refines D10 (a missing dependency row is not by itself
  evidence of dead code) and settles F8 and F13.

  Links: area The stack · draws on engine-split D10, engine-split D14 · plans engine-split P0, engine-split P3 · summary A crate with no dependents stays unless its own docs say it is a superseded stopgap; gazette-ledger retires, ux-events and document-host stay.

- **D40. mere-edit goes to Eidetic's data root (Mark, 2026-11-09):** P0 F16. `mere-edit` is
  232 source lines, serde only, unpublished: "revision-bound UTF-8 edit proposals without
  parser, editor or storage authority". Its consumers are Knot's `knot-document` and Isocosm's
  pack author, both of which already name eidetic.git, and nothing in Mere uses it. Ruling:
  "4, 1 if that's not reasonable". It is reasonable: no consumer gains a repository edge, and a
  revision-bound proposal is data that both sides link (Stack 9). It moves to Eidetic under
  its own name. Outcome: applies D22, and keeps D19's contract root from taking another leaf.

  Links: area The stack · draws on engine-split D22, engine-split D19 · plans engine-split P3 · summary mere-edit moves to Eidetic as a data-root leaf; the Graphshell contract root does not grow.

- **D41. The app's linked modules leave by the door or the wire, not by extraction (Mark,
  2026-11-09):** P3 Cut A. The reference app (`ports/graphshell`, about 105k lines) cannot go
  to Turnstone first, because `mere-resident` links 13 of its modules (81 use lines in 19
  files, about 23k lines), and Cleromancy (`personal_sync`) and Woodshed (`block_editing`,
  `capsule_applet`) link it too. Offered an extracted hosting crate, door and wire, or parking
  the app in Graphshell. Ruling: "Over the door or wire". The resident and the outside consumers
  reach the app through the door or the wire instead of linking it. That is the
  resident split plan (`graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-05_resident_split_plan.md`)'s ground, so P3's slices past the
  first wait on it. Outcome: new; orders P3 after the resident's door and wire work.

  Links: area The stack · draws on engine-split D23 · plans engine-split P3 · summary The resident, Cleromancy and Woodshed stop linking the reference app and reach it over the door or the wire; P3's toolkit and canvas moves wait on that.

- **D42. statements.rs retires (Mark, 2026-11-09):** P0 F2. `linked-data/src/statements.rs`
  (309 lines plus 625 of tests) is the engine's only edge to `inker`, and its apply half has no
  caller in ten repositories. Ruling: "Retire it (Recommended)". The file and `mere-rdf`'s
  `inker` row are deleted (git keeps them). Inker keeps the `link_statements` walk, and P5
  writes apply against the dataset. With D34, the engine and Inker name nothing of each
  other. Outcome: settles F2 and corrects P1's "statements walk the engine ingests".

  Links: area The stack · draws on engine-split D34 · plans engine-split P1, engine-split P5 · summary The unused statements apply module retires; mere-rdf drops its inker edge.

- **D43. The session manifest splits at the type (Mark, 2026-11-09):** P0 F21.
  `GraphSessionManifest` has 22 fields, three of them host-flavoured (`active_workers`,
  `engine_profile`, `policy`). `WorkerKind`, `SessionPolicy` and pandect's `EngineProfileBinding`
  enum have no user outside pandect. The lifecycle reads the manifest, so moving it whole would be
  a cycle. Ruling: "Split at the type (Recommended)". Lineage, marks and the codicil fields stay. The
  three host fields become one opaque `#[serde(default)]` extension slot that pandect round-trips
  without naming. `WorkerKind`, `SessionPolicy`, `session_service_runner` and
  `engine_profile_store` go to the host. Outcome: applies P4 ("worker declarations ... most to
  the host") and keeps room reserved for the session-service-runner plan.

  Links: area The stack · draws on engine-split D3 · plans engine-split P4 · summary The session manifest keeps lineage and marks; host fields ride an opaque extension slot; worker and policy types go to the host.

- **D44. The view stream keeps its log in pandect with an opaque payload (Mark, 2026-11-09):**
  P0 F24. `graph_session` stores `ViewIntent`, a host sidecar type naming incipit curation and
  sceno, in a stream written atomically with the journal batch. Ruling: "Opaque payload
  (Recommended)". Pandect keeps `ViewKey` and the log beside the journal, with a
  schema-referenced value as the payload. `ViewIntent` and `view_intent_store` move to
  Graphshell, which defines and edits them. Outcome: applies P4 and domain contract C7.

  Links: area The stack · draws on engine-split D3 · plans engine-split P4 · summary Pandect keeps the view log with an opaque payload; ViewIntent and its store move to Graphshell.

- **D45. The furnace splits by nature: retention to Eidetic, consolidation stays (Mark,
  2026-11-09):** P0 F22. Asked first, Mark answered: "this feels less like graph and more like
  dealing with stm in eidetic, no? What does furnace do?" The code (`pandect/src/furnace.rs`,
  842 lines) is housekeeping, not distillation. It has four propose/apply passes:
  - forgetting evicts the cached page bodies of untagged short-term nodes;
  - image GC drops unreferenced preview blobs;
  - consolidation composes the lineage of saved codicils that cover the same material;
  - retirement empties aged tombstones.
  `memory_levels` (404 lines) classifies nodes as short-term or long-term. The apply halves call
  Eidetic's `content_store`, `purge_deleted` and retention. Ruling: "Split by nature
  (Recommended)":
  - Retention goes to Eidetic's memory program: `memory_levels`, made generic over node records
    with no kernel types, plus forgetting's apply and retirement.
  - Consolidation, which is codicil lineage, stays in pandect as lifecycle.
  - The graph-reading propose halves stay as thin pandect adapters.
  Outcome: refines distillery-extraction D12 (graph-reading stays with the graph, retention
  mechanics go to the store's memory program). Weigh it with the Eidetic squish F4, still open,
  since it grows `eidetic-core`'s product half.

  Links: area The stack · amends distillery:distillery-extraction D12 · plans engine-split P4 · summary Memory levels, forgetting and retirement go to Eidetic; codicil consolidation stays in pandect; graph-reading proposals are pandect adapters.

- **D46. Session distillation is reserved in Distillery (Mark, 2026-11-09):** Mark's account
  of the furnace's purpose is winnowing browsing sessions into an indexed slice to inform a
  model or agent adapter, for example one offered to a moot's FLoRA round. That exists nowhere in
  code. Gemot's `moot/flora.rs` holds round contracts and "does not execute training", and the
  2026-06-09 Alembic doc's "mints distillates" went with the workshop to Distillery unbuilt.
  Ruling: "Distillery, reserved now (Recommended)". It is a known future, not P4 work. A
  Distillery job reads Eidetic captures, codicils and search, and emits a dataset engram that a
  FLORA round can cite. Distillery's plans carry the reservation. Outcome: new; applies
  "design for known futures".

  Links: area The stack · draws on engine-split D45 · summary Distilling sessions into dataset engrams for adapters is Distillery's reserved job, reading Eidetic and cited by Gemot's FLORA rounds.

- **D47. The preview-image store goes to Eidetic beside content_store (Mark, 2026-11-09):**
  P0 F23. Asked first: "is it the graph's images, or images saved in sessions?" It is the
  graph's node preview imagery: favicons, page previews and last-render snapshots. Kernel nodes
  hold an `ImageRef`, and the pixels live content-addressed at `content/image/<blake3>`. It is
  the sibling of `eidetic::content_store`. Session content images are Turnstone's own sidecar.
  Ruling: "Store half to Eidetic (Recommended)". Save, load, delete and list by digest (about
  130 lines) go to Eidetic. The orphan check stays with pandect's graph-reading propose, and
  `migrate_legacy_images` goes to the legacy snapshot adapter. Outcome: applies D45.

  Links: area The stack · draws on engine-split D45 · plans engine-split P4 · summary The content-addressed preview-image store moves to Eidetic; pandect keeps the graph-aware orphan check.

- **D48. nisus and modulus stay, because each has more to become (Mark, 2026-11-09):** P2
  fork. Mark's rule: "I am ok with 1 if there is more nisus can become, or 2 if not. Same with
  modulus." Both have a ruled future:
  - nisus (635 lines): Isocosm wing rulings 330 ("nisus grows into the voxel authority") and
    697 (the store behind Isometer), and the conatus engine plan's T2, make it the revisioned
    voxel world store.
  - modulus (2,239 lines): the product-neutral sparse-brick GPU ABI and the exact CPU walk that
    Isometer calls (wing 361) and Eponym draws through.
  Both stay whole. Outcome: applies D10's "one intention per crate" (each has its own); the P2
  nisus fork is closed.

  Links: area The stack · draws on engine-split D10 · plans engine-split P2 · summary nisus (the voxel authority to be) and modulus (the brick ABI) each keep their own crate.

- **D49. numen splits: field definitions go to chartulary, evaluation stays in Seiche (Mark,
  2026-11-09):** P2 fork, `graph-kernel`'s `numen` edge. Offered the workspace half, the engine
  half, or a split. Mark: "Wait, why wouldn't we do 3? I figure the definitions for the three
  should be together… and fields becoming meaningful in an rdf sense could be done,
  potentially. If nothing else, then with querying and scripting, no?" Nothing argues against
  it except its size: a crate split across three repositories, and `numen` is published.
  - numen's definition half (about 930 lines: `field`, `coupling`, `field_ast`, `edge_path`)
    moves down to Eidetic's `chartulary`, beside the generic nodes and relations and its
    existing RDF projection.
  - The evaluator stays in Seiche.
  - Neither engine nor host names Seiche for field truth.
  Link-model L9 (fields act as forces, never as assertions) stands. A field becoming queryable
  or scriptable through the RDF projection is a later P6 or P7 question, not an assertion.
  Outcome: new; applies Stack 9 (a leaf goes to the root of its nature).

  Links: area The stack · draws on engine-split D22 · plans engine-split P2, engine-split P5 · summary Field definitions join nodes and edges in chartulary; numen's evaluator stays in Seiche; fields may become queryable through chartulary's RDF projection later.

- **D50. eidetic-core's product half is feature-gated in place (Mark, 2026-11-09):** Eidetic
  squish F4. `eidetic-core` is 16,557 lines, about 65% of them browsing-memory product
  (`browsing`, `models`, `import`, `records`, `capture` and smaller ones), compiled by 42
  manifests in 11 repositories, while Gemot and Dramatis use substrate paths only. D45 and D47
  add more product. Ruling: "Feature-gate in place (Recommended)". The product modules sit
  behind features, and substrate-only consumers set `default-features = false`. No name is
  spent and no ruling is reversed. Outcome: applies memory-gates D17 and D10; settles F4.

  Links: area The stack · draws on engine-split D45, engine-split D47 · plans engine-split P4 · summary eidetic-core gates its browsing-memory modules behind features; substrate-only consumers opt out.

- **D51. mere-overmap and mere-view stay separate (Mark, 2026-11-09):** P0 F3. Two hosts take
  overmap's `wire` without the view: the Graphshell port, and the headless `mere-resident`.
  `mere-view` carries Cambium, cartography, pictograph and sprigging as non-optional
  dependencies. Ruling: "Keep both (Recommended)". Both move with P3. Outcome: applies D10's
  test (overmap's job is not one module of the view's).

  Links: area The stack · draws on engine-split D10 · plans engine-split P3 · summary overmap keeps its wire crate apart from the view so the headless resident avoids the Cambium cone.

- **D52. The three small script hosts merge into app-host (Mark, 2026-11-09):** P0 F12. There
  are four wasm hosts, each repeating `wasmtime = "45"`, a `build.rs` and a guest: `app-host`
  (483 lines), `block-host` (273), `authority-host` (411) and `document-host` (3,722, which D39
  keeps). Ruling: "Three small into app-host (Recommended)". `block` and `authority` become
  features of `app-host`, sharing one wasmtime pin. `document-host` stays separate with its own
  Genet and Inker cone. Frees `block-host` and `authority-host`. Outcome: applies D10.

  Links: area The stack · draws on engine-split D10, engine-split D39 · plans engine-split P3 · summary block-host and authority-host become features of app-host; document-host stays its own crate.

- **D53. uxtree grows into the stack's accessibility and automation spine (Mark, 2026-11-09):**
  P0 F6. Asked first: "Is there more that can be done with it to support accessibility,
  diagnostics, and automation stack wise?" uxtree projects documents and workbenches into
  AccessKit with stable path-derived IDs. Its `graft` joins independent trees, and `frozen`
  catches unlabelled nodes. Turnstone's a11y layer (25 files, 1.3k lines) sits on it, while
  Cambium (sprigging, rootstock) and Genet's taproot (the scenario driver formerly named
  genet-probe) build their own AccessKit trees. Ruling: "Grow into the spine (Recommended)".
  It stays whole and gets a brief, with no code now, covering four directions:
  - graft joins the Cambium and taproot subtrees into one ID space;
  - taproot scenarios address native surfaces by uxtree path, one automation grammar for both;
  - a lint pass (unlabelled, orphaned and unstable IDs) asserts through `mere-registry`'s
    diagnostics;
  - Turnstone's generic a11y pieces are audited for promotion.
  Outcome: new; refines the 2026-11-07 finding that uxtree stays whole, and gives it a
  direction. The brief is written (2026-11-11): `graphshell/design_docs/graphshell_docs/research/2026-11-11_uxtree_spine_brief.md`; its nine forks wait for Mark. Mark's rulings on the briefs are recorded as `graphshell:uxtree-spine D1` to `D9` in the plan at
  `graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-11_uxtree_spine_plan.md`; D8 there deletes the kernel's `descriptor_for`.

  Links: area The stack · draws on engine-split D10, engine-split D26 · plans engine-split P3 · summary uxtree becomes the one accessibility and automation tree: grafted subtrees, path-addressed automation, invariant lints through diagnostics.

- **D54. query_view moves into mere-view (Mark, 2026-11-09):** P0 F10. The `mere` facade's
  `query_view` (892 production and 727 test lines) sits behind a `query` feature that pulls in
  SPARQL, yet it uses none. It applies views over `kernel::Graph` and `incipit::forme`, and
  its consumers are mere-view's query editor and the Graphshell port. Ruling: "Into mere-view
  (Recommended)". It moves into `mere-view` as a module, `mere-view` drops its `mere` edge, and
  the port's `query-views` feature repoints. Outcome: applies the plan's facade dissolution
  and keeps the engine's Query stage meaning SPARQL.

  Links: area The stack · draws on engine-split D1 · plans engine-split P3 · summary query_view goes with its editor into mere-view; the engine's query stage is SPARQL only.

- **D55. mere-moot stays a host port (Mark, 2026-11-09):** P0 F14. Ruling: "Stays a host port
  (Recommended)". Gemot stays presentation-free, and the port moves with P3. Outcome: applies
  the Layering constraint.

  Links: area The stack · plans engine-split P3 · summary mere-moot is a Graphshell host port; Gemot names no host crate.

- **D56. Alembic merges into Athanor, which stays with the host (Mark, 2026-11-09):** P0 F15
  and P3 F4. Athanor (5,243 lines) depends on Alembic (747), its workshop surface. Athanor's
  `workshop-destination` names pandect and the kernel, and moving Alembic alone would make a
  Graphshell→Distillery→Graphshell triangle. Ruling: "Merge, home unchanged (Recommended)".
  Alembic folds into `mere-athanor` behind features, and the merged crate moves with the host
  in P3. Any later move to Distillery is one move. Frees `mere-alembic`. Outcome: refines the
  plan's "mere-alembic to Distillery"; applies D10.

  Links: area The stack · draws on engine-split D10, distillery:distillery-extraction D12 · plans engine-split P3 · summary mere-alembic folds into mere-athanor, which moves with the host; a Distillery move later is one move.

- **D57. djinn-testkit stays with the resident (Mark, 2026-11-09):** P0 F11. Its one dependent
  is mere-resident's tests, and it spawns the resident's own `djinn` binary. Ruling: "Stays with
  the resident (Recommended)". It moves beside `mere-resident`, not to Armillary. Outcome:
  confirms the dispositions list; P0's "to Armillary" line is withdrawn.

  Links: area The stack · plans engine-split P3 · summary djinn-testkit tests the resident and moves with it.

- **D58. The mere facade dissolves in P3's third slice (Mark, 2026-11-09):** P3 F6. The facade
  (6,928 lines) is named about 970 times across Mere, Turnstone and Cleromancy, mostly as
  re-exports, and P5 reuses the name `mere`. Ruling: "Dissolve in slice 3 (Recommended)".
  Navigation, trail and roster go to incipit, and consumers name the kernel, cartography and
  the rest directly. The name is free for P5's engine. Outcome: applies P3 as written and D54.

  Links: area The stack · draws on engine-split D8, engine-split D54 · plans engine-split P3, engine-split P5 · summary The mere facade dissolves when the toolkit moves; the name passes to the engine crate.

- **D59. graph-kernel's split waits for P8 (Mark, 2026-11-09):** P3 F7. The kernel's "surface
  half" is not a module set (`kernel::graph` alone is used 631 times by host crates), and the
  kernel has no Mere dependencies. Ruling: "Wait for P8 (Recommended)". P3 moves no kernel
  code, hosts name `mere-kernel` from mere.git, and the split follows the engine-to-workspace
  contract. Outcome: applies D4.

  Links: area The stack · draws on engine-split D4 · plans engine-split P3, engine-split P8 · summary P3 moves no kernel code; hosts take mere-kernel from mere.git until the P8 contract.

- **D60. pandect's edges to incipit and tabard are cut in P3 (Mark, 2026-11-09):** P3 F8.
  pandect names incipit (`SessionId` and curation folds, in 11 store files) and tabard
  (`ThemeChoice` and `Mode` in its settings stores). These are the engine's only edges into
  Graphshell-bound crates, and they pin both crates in Mere. Ruling: "Cut in P3", over holding
  until P4. The settings and curation store work of P4 (D43, D44) is pulled forward far enough
  to cut both edges, so incipit and tabard can leave with the toolkit rather than in a slice
  after P4. Outcome: reorders P3 and P4; P3's slice 4 is no longer gated on P4.

  Links: area The stack · draws on engine-split D43, engine-split D44 · plans engine-split P3, engine-split P4 · summary P3 cuts pandect's incipit and tabard edges so both leave with the toolkit.

- **D61. The conformance ledger scores two layers (Mark, 2026-11-09):** ledger fork F2. The
  first run (`mere_docs/testing/2026-11-09_conformance_ledger.md`) recorded two layers. The
  parser layer is the Oxigraph component crates as the adapter configures them. The adapter
  layer is `mere-rdf`'s own entry points, scored accept or reject, and it surfaces gaps such as
  TriG 1.1 at 199/214 because `from_trig` takes no base IRI. Ruling: "Both layers
  (Recommended)", marked provisional until P5's engine API. Outcome: refines the Constraints
  ledger columns.

  Links: area The stack · plans engine-split P0, engine-split P5 · summary Ledger rows record both the parser layer and the adapter layer until the engine API exists.

- **D62. The ledger runs SPARQL evaluation and RDF/XML now (Mark, 2026-11-09):** ledger fork F3.
  Asked first: "I don't mind either, but costs?" `sparesults` is already in Mere's lock (0.3.5,
  through spareval), so the 341 SPARQL 1.1 evaluation tests cost harness code only. `oxrdfxml`
  is about one new crate over `oxrdf` and `quick-xml`, both present, and the Formats stage names
  RDF/XML. Ruling: "Both (Recommended)". SHACL waits for P6. Outcome: applies "Standards follow
  the suites".

  Links: area The stack · draws on engine-split D61 · plans engine-split P0 · summary The ledger adds SPARQL evaluation through sparesults and RDF/XML through oxrdfxml; SHACL waits for P6.

- **D63. The ledger's first-run choices stand (Mark, 2026-11-09):** ledger forks F4 to F6.
  Ruling: "Accept all three (Recommended)". The RDF 1.1 baseline and SPARQL 1.2 rows stay, since
  rdf12 is only a delta over 1.1. Spec revision is the suite directory plus the clone HEAD,
  since the suites carry no dated snapshot. The file keeps its dated name. Outcome: applies P0.

  Links: area The stack · plans engine-split P0 · summary The ledger keeps its RDF 1.1 baseline rows, records suite directory and clone revision, and keeps its dated name.

- **D64. murm-ocapn-transport becomes a feature of murm-ocapn (Mark, 2026-11-09):** asked
  whether Murm needs squishing, the evidence lane audited its 39 packages under D10 and found
  one real candidate. `murm-ocapn-transport` (294 lines) admits OCapN onto an existing
  transport, owns no endpoint, discovery or keys, and has one dependent, a Mere testing probe.
  Ruling: "Merge as a feature (Recommended)". It becomes murm-ocapn's `native` feature and frees
  its name, done in Murm on Murm's schedule. The rest of Murm passes D10, and its crates with no
  dependents are kept under D39. Outcome: applies D10 and D39.

  Links: area The stack · draws on engine-split D10, engine-split D39 · summary Murm needs one squish: murm-ocapn-transport folds into murm-ocapn as a feature.

- **D65. SPARQL update evaluation waits for the engine's update stage (Mark, 2026-11-09):** ledger fork F7. spareval 0.2.9 evaluates queries and DELETE/INSERT-WHERE deltas only, and no dependency executes INSERT DATA, DELETE DATA, LOAD, CREATE, CLEAR, DROP, ADD, COPY or MOVE, so running the 97 update tests (94 in 1.1, 3 in 1.2) would score a harness executor, not the engine. Ruling: "Not run until the engine (Recommended)". The rows read "not run: no update stage" until P5 or P6 adds one. Outcome: applies D61.

  Links: area The stack · draws on engine-split D61 · plans engine-split P5 · summary SPARQL update evaluation is not run until the engine has an update stage.

- **D66. A move re-addresses its consumers' git rows; no pins, no version addressing (Mark,
  2026-11-09):** P3 slice 1 moved `tabard-workshop` to graphshell.git, leaving seven rows in
  Woodshed, Cleromancy and Murm that named mere.git. Those rows already followed the
  cycle-breaks rule (`branch = "main"`, no `rev`); the break is the address, because a git row
  names the repository Cargo fetches from. Offered version addressing (crates.io or a family
  registry) so that moves become invisible, or keeping git rows and fixing them per move. Mark:
  "we should have the names either way. for when they deserve stable releases. readdress the
  repos. no pins. although it would probably be easier to just do cargo bumps and make all the
  crates name a minimum acceptable version, but somehow i'm sure that will become a monkey's
  paw". Each move rewrites its consumers' rows to the crate's new canonical URL, still
  `branch = "main"`, as part of the same phase. Version requirements are not used for family
  crates. Crate names are claimed on crates.io for when a crate earns a stable release, which
  is not a way of addressing it. Outcome: applies the cycle-breaks row rule; P3's later slices
  carry their consumer re-addressing as a done-condition.

  Links: area The stack · plans engine-split P3 · summary Moving a crate rewrites every consumer's git row to its new repository, branch main, no pins; family crates are not version-addressed; names are claimed for future stable releases.

- **D67. The app's linked modules dissolve into their owners; D41's door and wire is withdrawn
  (Mark, 2026-11-09):** D41 ruled that the resident and outside consumers reach the reference
  app "over the door or wire". The evidence lane (receipt file, 2026-11-09) found the premise
  wrong for the resident. The resident serves all 13 linked modules and never calls the app:
  `MereHost` is the resident's own `mere` route provider, `receipts` says "authoring is the
  resident host's alone", and coop's owner side runs in the resident. There is no app process
  for it to call. Put back as dissolve into owners, literal routes, or the hosting crate D41
  declined. Ruling: "Dissolve into owners (Recommended)". Each module moves to the owner below
  both sides, the way Turnstone dropped the package on 2026-10-15. No hosting crate and no new
  routes; the app keeps reaching the resident over the existing doors. Older rulings in the
  way: "Supersede both (Recommended)". Stack-lift's "Group 4 product truth is not lifted" and
  key-lane D4's "the personal-graph event types travel with `ports/graphshell`" are both
  superseded here. Outcome: replaces D41; applies domain contract C5; P3's later slices no
  longer wait on the resident split plan.

  Links: area The stack · replaces engine-split D41 · amends graphshell:key-lane D4 · draws on engine-split D23 · plans engine-split P3 · summary The modules the resident, Cleromancy, Woodshed and Knot link from the app move to owners below both sides; no new routes; stack-lift Group 4 and key-lane D4's event-types clause are superseded.

- **D68. The mere endpoint closure is a feature of graphshell-host (Mark, 2026-11-09):** D67's
  F3. `mere_host` (2,467 lines), `handlers` (328), `session_item` (81) and `resident_host` (305)
  are used by the resident and by about 60 app files, and implement `graphshell-endpoint` traits
  over chirograph, sceno and scenograph. Ruling: "graphshell-host feature (Recommended)". They
  become a `mere-host` feature of `graphshell-host`, below both the app and the resident.
  Outcome: applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary MereHost and its handlers live in graphshell-host behind a mere-host feature.

- **D69. personal_sync moves to pandect now, as it is (Mark, 2026-11-09):** D67's F4.
  `personal_sync` (about 4.7k lines) names facets 54 times while the facet-dissolution brief is
  open, and key-lane D1 is removing `PublishPrekey` and `GroupDispatch` from it. Ruling: "Move
  as-is now (Recommended)". It moves with `access` (581 lines), and the facet and key-lane
  rulings land in pandect afterwards. Cleromancy then builds with no `graphshell` package.
  Outcome: applies domain contract C5 and D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3, engine-split P4 · summary personal_sync and access move to pandect now; later facet and key-lane rulings land there.

- **D70. Transfer's data types and offer decode move down; product stays the app's (Mark,
  2026-11-09):** D67's F5. The resident's production code uses transfer's data types and
  `offer_for`, which calls `product::decode_profiled_codicil`. Ruling: "Data types + offer
  decode move (Recommended)". The data types, `transfer_offer` and the codicil decode move with
  `personal_sync`. `product` stays the app's data source, and the resident's tests that called
  `prepare_transfer` and `apply_transfer` are rewritten. Outcome: applies D67.

  Links: area The stack · draws on engine-split D67, engine-split D69 · plans engine-split P3 · summary Transfer data types and the offer decode move below the resident; product stays in the app.

- **D71. kept_text goes to Eidetic; the goal-surface provider goes to the resident (Mark,
  2026-11-09):** D67's F6. `kept_text` (245 lines) imports only eidetic, muniment, serde and
  url. The goal surface's resident half (provider, facts and door, 713 lines) also imports
  Fleece's retained traversal, so Eidetic cannot hold it. Ruling: "kept_text to Eidetic,
  provider to resident (Recommended)". The roughly 2.9k-line viewer stays in the app. Outcome:
  applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary kept_text's intents, policy and writer join Eidetic; the goal-surface provider becomes resident code; the viewer stays in the app.

- **D72. Woodshed's conformance test runs over block-host (Mark, 2026-11-09):** D67's F7. The
  test loads the signed pack through the app's `block_editing`, `capsule_applet` and
  `graphshell-blocks`, a dev row that becomes a cycle once the app is in Turnstone. Ruling:
  "Rewrite over block-host (Recommended)". It tests over the `block-guest` WIT world with
  Eidetic's pack review, and `GuestAdapter` and `GuestEngine` move into an app-agnostic crate.
  Outcome: applies D67 and cycle-breaks D6.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary Woodshed tests its pack over the block-guest wire, with no link to the app.

- **D73. BlockEditor moves to cambium, message-shaped (Mark, 2026-11-09):** D67's F8. Offered
  the Knot adapter moving to Turnstone, the trait moving to cambium, or a wire guest. Mark: "my
  instinct is 2 but... i mean, would 3 impart modifiability in some manner, or enforce a
  correct boundary we'd otherwise have to derive?" It would enforce one boundary: five of the
  trait's seven methods are already messages, and only `input()` and `input_mut()`, which hand
  out Cambium's `TextInput`, couple it in-process. Ruling: "Cambium, message-shaped
  (Recommended)". The trait moves to cambium without the two widget accessors, and the pane
  owns its `TextInput`. That is the wire's boundary kept in-process, and a WIT editor guest is
  a mechanical wrap later. Outcome: new; applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary BlockEditor is a cambium trait of messages only; the pane owns the text widget; a wire guest can wrap it later.

- **D74. catchment is generic over the author of an access decision (Mark, 2026-11-09):** P4
  F1. The access record named `kernel::graph::Author`, which would put the kernel in
  catchment's path and fail P4's done-condition. Ruling: "Generic over author (Recommended)".
  `MereApplicationAccess<A>` and `ReservoirStore<B, A>` are generic, and pandect aliases them
  with `Author`. Stored bytes and consumers are unchanged. Outcome: applies D3 and D9.

  Links: area The stack · draws on engine-split D3, engine-split D9 · plans engine-split P4 · summary catchment's access types are generic over the author; pandect supplies the kernel's Author.

- **D75. SessionId and GraphId live in catchment (Mark, 2026-11-09):** P4 F4, for D60. pandect
  uses incipit's ids in about 100 places and keys the manifest by them. Ruling: "catchment
  (Recommended)". The ids move into catchment, below both pandect and incipit, and incipit
  re-exports them. incipit's default then carries catchment rather than only serde and uuid.
  Outcome: applies D60; refines D8's "ids behind a minimal default feature".

  Links: area The stack · amends engine-split D8 · draws on engine-split D60 · plans engine-split P4 · summary Session and graph ids are catchment's; incipit re-exports them; pandect no longer names incipit for ids.

- **D76. chartulary takes uuid's v4 and js features, and no strum (Mark, 2026-11-09):** D49's
  fork. numen's definitions need `strum` for one derive and uuid's `v4` for random ids, neither
  of which chartulary has. Ruling: "uuid features, no strum (Recommended)". chartulary's
  existing uuid gains `v4` (native) and `js` (wasm), and the derive becomes a hand-written
  `iter()` of the same signature. Outcome: applies D49.

  Links: area The stack · draws on engine-split D49 · plans engine-split P2 · summary chartulary widens uuid's features for field ids and hand-writes the one iteration strum provided.

- **D77. D43's host types split by job (Mark, 2026-11-09):** P4 F2. Nothing outside pandect
  uses `WorkerKind`, `SessionPolicy`, `session_service_runner` or `engine_profile_store`.
  Ruling: "Split by job (Recommended)". The runner, `WorkerKind` and `SessionPolicy` go to a
  `mere-resident` module. `engine_profile_store` and its `EngineProfileBinding` go to
  `mere-fetch`, beside the fetch service's per-engine site data. Outcome: applies D43.

  Links: area The stack · draws on engine-split D43 · plans engine-split P4 · summary The session service runner and its types go to the resident; the engine profile store goes to mere-fetch.

- **D78. ViewIntent lives in incipit behind a view feature (Mark, 2026-11-09):** P4 F3, for
  D44. `ViewIntent` and `view_intent_store` (602 lines) name incipit's `FoldRecord`, sceno's
  `LinkKey` and hest's `CommandChoices`, and crates below the port use them. Ruling: "incipit
  `view` feature (Recommended)". They move to incipit behind a `view` feature that carries
  sceno and hest, and pandect drops those two edges. Outcome: applies D44 and D8.

  Links: area The stack · draws on engine-split D44, engine-split D8 · plans engine-split P4 · summary ViewIntent and its store are incipit's view feature; pandect stops naming sceno and hest.

- **D79. The settings theme is stored opaque, with the same bytes (Mark, 2026-11-09):** P4 F5,
  for D60. `ApplicationSettings` held `tabard::ThemeChoice`, and its only outside consumer is
  Turnstone. Ruling: "Opaque theme, same bytes (Recommended)". The store stays in pandect and
  holds the theme as a plain id, mode key and fonts with identical serialization. Turnstone
  converts to `ThemeChoice` at its edge. Outcome: applies D60.

  Links: area The stack · draws on engine-split D60 · plans engine-split P4 · summary pandect stores the theme as plain values; Turnstone converts to tabard's type.

- **D80. subgraph moves into mere-view with query_view, now (Mark, 2026-11-09):** D54's
  premise was wrong. `query_view` uses no SPARQL itself, but `QueryViewSpec::members` calls
  `mere::subgraph::try_derive_members`, which evaluates saved SPARQL through linked-data's
  `query` feature. Offered deferral to slice 3, moving subgraph now, or moving the module with
  the `mere` edge kept. Ruling: "Move subgraph now", over the recommended deferral. `subgraph`
  (556 lines with tests) moves into mere-view with `query_view`, and mere-view takes
  linked-data's `query` directly and drops `mere`. Outcome: refines D54.

  Links: area The stack · amends engine-split D54 · draws on engine-split D58 · plans engine-split P3 · summary subgraph derivation and query_view both live in mere-view, which names linked-data's query directly and not the mere facade.

- **D81. carriage and receipts become resident code (Mark, 2026-11-09):** D67's F11.
  `carriage` (608 lines) has no app consumer, and `receipts` (1,637) says "authoring is the
  resident host's alone". Ruling: "Resident sync/ (Recommended)". Both move into
  mere-resident's `sync/`, beside `carriage_host`; the key-lane plan's later changes land
  there. Outcome: applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary carriage and receipts move from the app into the resident's sync module.

- **D82. The resident's MCP test moves to Turnstone with graphshell-mcp (Mark, 2026-11-09):**
  D67's F10. mere-resident names `graphshell-mcp` as a dev dependency for one test, and D23
  sends that crate to Turnstone. Ruling: "Port to the MCP crate's home (Recommended)". The test
  becomes a Turnstone consumer test over the resident's test-fixtures export. Outcome: applies
  D23 and cycle-breaks D6.

  Links: area The stack · draws on engine-split D23, engine-split D67 · plans engine-split P3 · summary The MCP test over the resident lives in Turnstone beside graphshell-mcp.

- **D83. The dissolve work is P3's slice 2 (Mark, 2026-11-09):** D67's F9. The resident split
  plan moves different pieces in the other direction and does not mention the app link.
  Ruling: "Engine split P3, slice 2 (Recommended)". The steps S1 to S6 are written under P3,
  and the resident split plan's constraint on `graphshell::` types notes that slice 2 removes
  those paths. Outcome: applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary The dissolution of the app's linked modules is written and tracked as P3 slice 2.

- **D84. slot_guard moves into catchment (Mark, 2026-11-09):** P4 F6. `slot_guard` (170 lines,
  compare-and-replace over a muniment slot) is used only by four Graphshell port files. Offered
  leaving it in pandect until P3, catchment, or Eidetic's muniment. Ruling: "Move to
  catchment", over the recommended wait. It sits behind catchment's `store` feature (D85).
  Outcome: applies the P0 audit's placement.

  Links: area The stack · draws on engine-split D9 · plans engine-split P4 · summary slot_guard is catchment's, behind its store feature.

- **D85. catchment's default is ids only; the reservoir sits behind a store feature (Mark,
  2026-11-09):** D75 landed (`b67021f0c`) and, measured, took incipit's default from serde and
  uuid to 168 crates: catchment brings eidetic, muniment, dramatis.git's personae and sporran,
  and their crypto. The cost put to Mark for D75 had understated it. incipit is published and
  catchment is not, and D8 keeps the ids "behind a minimal default feature". Offered gating
  catchment, the ids in Eidetic, the ids back in incipit, or accepting the weight. Ruling:
  "Gate catchment (Recommended)". catchment's default holds `SessionId` and `GraphId` over
  serde and uuid. The catalog, store, sealing and custody sit behind `store`, which pandect and
  the resident enable. catchment publishes before incipit's next release. Outcome: refines
  D75; restores D8's minimal default.

  Links: area The stack · amends engine-split D75 · draws on engine-split D8 · plans engine-split P4 · summary catchment's default feature is the ids alone; the reservoir, sealing and custody are its store feature; incipit's default stays light.

- **D86. live_view moves to graphshell-host (Mark, 2026-11-09):** P4 F7. `pandect::live_view` (431
  lines) embeds `ViewIntent` in a persisted `LiveViewRecord` and validates its folds, and has no
  consumer in any repository. While it stayed, pandect still named incipit's view feature, sceno
  and hest. Offered graphshell-host, retirement, an opaque payload in pandect, or incipit's view
  feature. Ruling: "graphshell-host feature (Recommended)". It moves to the host side behind a
  feature, where the P0 audit placed it, and is kept under D39. Outcome: applies D44, D60 and D78.

  Links: area The stack · draws on engine-split D44, engine-split D78, engine-split D39 · plans engine-split P4 · summary live_view is a graphshell-host feature; pandect no longer names incipit's view feature, sceno or hest.

- **D87. The retention passes are eidetic::memory (Mark, 2026-11-09):** D45's placement. Eidetic
  already has `eidetic::retention`, a different subject (owner-scoped retained copies with custody
  and quota; it never evicts). The moved code classifies short-term and long-term records, drops
  cached content and purges tombstones. Offered submodules of `retention`, a new `memory` module,
  or another name. Ruling: "eidetic::memory (Recommended)". The modules are
  `memory::{levels, forgetting, retirement}` behind a `memory` feature that needs `content_store`
  and `deleted` and is part of the default-on `product`. Outcome: applies D45 and D50's one feature
  per module.

  Links: area The stack · draws on engine-split D45, engine-split D50 · plans engine-split P4 · summary Memory levels, forgetting and retirement live in eidetic::memory behind a memory feature; eidetic::retention stays the custody module.

- **D88. The whole of coop moves to graphshell-host (Mark, 2026-11-09):** Slice 2's S1 sent
  "`coop`'s owner side" to `graphshell-host`. The S1 lane found no separable owner side. Of
  coop's 1,508 non-test lines, the shared types (`Data`, `Edit`, `Operation`) and `Owner` sit
  together in `mod.rs` (702), `network.rs` (536) mixes the owner's `host` with a 207-line
  `Guest`, and one 527-line test file shares a fixture across all of it. The app uses the owner
  too (`graphshell_coop` calls `Owner::open`), and the resident uses `Owner`, `control` and
  `network`. Offered the whole module, a split leaving `Guest` in the app, or a crate of its
  own. Ruling: "Whole coop to graphshell-host (Recommended)". All of `coop/` moves behind a
  `coop` feature of `graphshell-host`, and the app and the resident both import it from there.
  Outcome: refines S1's coop line; applies D67.

  Links: area The stack · draws on engine-split D67 · plans engine-split P3 · summary All of coop, guest and shared tests included, lives in graphshell-host behind a coop feature.

- **D89. The provenance tooling label is the caller's (Mark, 2026-11-09):** S2 moved `access`
  to pandect as it was (D69). It stamps stored records with a tooling label built from the
  crate version, `graphshell/{version}` and `graphshell-access/{version}`, so new records read
  `graphshell/0.2.0` (pandect's version) where they read `graphshell/0.0.2`. Schema ids are
  unchanged. Offered the caller passing it, relabelling to `pandect/`, or leaving it. Ruling:
  "Caller passes it (Recommended)". pandect takes the tooling string as an argument, and the
  app and the resident each name themselves and their own version. Outcome: new; applies D69.

  Links: area The stack · draws on engine-split D69 · plans engine-split P3 · summary pandect's access takes the provenance tooling label from its caller; the app and the resident each name themselves.

- **D90. The block pane keeps one undo journal (Mark, 2026-11-09):** D73 took `input()` and
  `input_mut()` off `BlockEditor`, so the pane owns its text widget. The lane landed the
  widget's Ctrl+Z on the widget's own history, apart from the editor's journal: that history
  resets when the editor changes the text, and the Undo button takes back one flush of typing
  per step. Offered one shared journal, keeping it as landed, or a headed look first. Ruling:
  "One shared journal (Recommended)". Ctrl+Z in the widget goes to the editor as an undo
  message, the widget's local history is not used, and a coalesced run of typing is one step
  as before. Outcome: refines D73.

  Links: area The stack · draws on engine-split D73 · plans engine-split P3 · summary The block pane's widget sends undo and redo to the editor; the editor's journal is the only history.

- **D91. The goal surface's provider, facts and door are a graphshell-host feature (Mark,
  2026-11-09):** D71 sent them to the resident. The lane found the app's viewer uses all three:
  `GoalSurface` holds `RetainedPages`, `suggest.rs` (326 lines) uses `GraphFacts` and
  `RetainedPages`, the web tree uses `Origin`, and `graphshell-mere` calls `door::from_wire`.
  In the resident they would make the app depend on the resident. Offered a graphshell-host
  feature, a crate of its own, or splitting the viewer out to reach pages over the door.
  Ruling: "graphshell-host feature (Recommended)". `provider`, `facts`, `door` and the fixture
  (about 800 lines) become a `goal-surface` feature of `graphshell-host`, below both the app and
  the resident; the viewer stays in the app and `kept_text` stays in Eidetic. Outcome: refines
  D71; applies D67.

  Links: area The stack · draws on engine-split D67, engine-split D71 · plans engine-split P3 · summary The goal surface's provider, facts and door live in graphshell-host behind a goal-surface feature, not in the resident.

- **D92. receipt_ingest moves to the resident with receipts (Mark, 2026-11-09):** D81 sent
  `receipts` to the resident's `sync/`. The lane found one app user that is not a test: the
  `receipt_ingest` bin (151 lines, `personal-sync`), the CLI `remote-receipt.ps1` calls to write
  receipts into the inbox. Offered both to the resident, `receipts` to pandect with the bin
  staying in the app, or retiring the bin. Ruling: "Both to the resident (Recommended)".
  `receipts` moves as D81 ruled, `receipt_ingest` becomes a resident bin beside it, and the
  script is repointed. Outcome: applies D81.

  Links: area The stack · draws on engine-split D81 · plans engine-split P3 · summary The receipt_ingest CLI is a resident bin beside receipts; receipt authoring stays in one place.

- **D93. The app's methods on MereHost become extension traits (Mark, 2026-11-10):** S3 moved
  `handlers` and `session_item` and stopped at `MereHost`. The app adds methods directly to it:
  14 in `product.rs` and 4 in `capture.rs`, called at 172 sites in 19 files, and Rust forbids
  that across crates. D70 keeps `product` in the app. The methods use four private primitives
  (`mutate_product_graph`, `set_facet`, `edit_candidate`, `commit_candidate`) and one private
  field. Offered extension traits in the app, free functions, or moving the methods down.
  Ruling: "Extension traits in the app (Recommended)". The app defines `ProductHost` and
  `CaptureHost`, implemented for graphshell-host's `MereHost`; call sites keep their shape and
  gain a `use`. The four primitives become `MereHost`'s public mutation interface. Outcome:
  new; applies D68 and D70.

  Links: area The stack · draws on engine-split D68, engine-split D70 · plans engine-split P3 · summary The app's product and capture methods on MereHost are extension traits over its public mutation primitives.

- **D94. The alembic workshop methods stay in the app (Mark, 2026-11-10):** `mere_host_alembic.rs`
  (519 lines, feature `alembic-workshop`) is 10 more methods on `MereHost` over athanor's
  workshop types, reading four private fields at 9 places. Only the app's `alembic_workshop`
  calls it; the resident does not. Offered staying in the app or moving with `MereHost` behind a
  graphshell-host feature that would name athanor and servitor. Ruling: "Stays in the app
  (Recommended)". It becomes app code over `MereHost` the way D93's methods do, reading the
  four fields through public accessors; graphshell-host gains no edge to athanor. Outcome:
  applies D93.

  Links: area The stack · draws on engine-split D68, engine-split D93 · plans engine-split P3 · summary The workshop methods on MereHost stay in the app as an extension over public accessors; graphshell-host names no athanor crate.

- **D95. The block adapter contract is a crate of its own (Mark, 2026-11-10):** D72 moved
  `GuestAdapter` and `GuestEngine` to "an app-agnostic crate". The lane found they implement
  the app's `BlockAdapter` trait and speak about 260 lines of its types (`Claim`, `Command`,
  `Field`, `Row`, `BlockInput`, `RunResult`, `ViewAsk`, `GuestView`), which need only sceno and
  vector-sheet, so the trait and types move with them. Offered a new crate or `block-host`.
  Ruling: "A new block-adapter crate (Recommended)". Trait, types and `GuestAdapter` live in
  `block-adapter`, over `block-host` and sceno; `block-host` stays a bare WIT host. Outcome:
  refines D72.

  Links: area The stack · draws on engine-split D72 · plans engine-split P3 · summary BlockAdapter, its types and GuestAdapter live in a block-adapter crate; block-host stays a bare WIT host.

- **D96. GuestAdapter::attach takes bytes and grants from its caller (Mark, 2026-11-10):**
  `attach` took the app's `CapsuleMount`, which cannot move down. Offered the caller passing
  bytes and grants, or a small review type moving with the adapter. Ruling: "Caller passes
  bytes and grants (Recommended)". The app reviews through `CapsuleMount` and Woodshed's test
  through Eidetic's `verify_pack`; the hash and `power:block-adapter` checks are the caller's.
  Outcome: applies D95.

  Links: area The stack · draws on engine-split D95 · plans engine-split P3 · summary attach takes component bytes and the granted powers; pack review is the caller's.

- **D97. The undo and paint checks stay with the app (Mark, 2026-11-10):** Without the app,
  Woodshed's conformance test keeps the claim, change-map and view-region checks and loses two
  that need app types: one undo restoring the document byte for byte (`Document`, `Preview`),
  and each view building a `SheetLeaf`. Offered an app-side test, dropping them, or a
  text-splice check. Ruling: "App-side test in Mere keeps them (Recommended)". They move to a
  test in the app's `graphshell-blocks` crate and travel with the app to Turnstone; Woodshed's
  test proves the wire. Outcome: applies D72.

  Links: area The stack · draws on engine-split D72 · plans engine-split P3 · summary Woodshed's test proves the block-guest wire; the document-undo and paint checks are an app-side test.

- **D98. gloss and glossary go to mere-view (Mark, 2026-11-10):** Slice 3's assessment found the
  facade at 3,522 lines. D58 sends `navigation`, `trail` and `roster` (2,206) to incipit; five
  modules had no ruled home. `gloss` (458 lines: outline snapshot, minimap geometry, the
  document outline into uxtree) needs pictograph, inker, tabard, netrender and accesskit;
  `glossary` (362: graph digests) needs only the kernel. Nothing outside the facade names
  `gloss`, and only `gloss` names `glossary`. Offered both to mere-view, splitting them, or
  retiring both. Ruling: "Both to mere-view (Recommended)", behind a `gloss` feature.
  Outcome: applies D58 and D54.

  Links: area The stack · draws on engine-split D54, engine-split D58 · plans engine-split P3 · summary gloss and glossary live in mere-view behind a gloss feature.

- **D99. coupling_edit goes to graphshell-host (Mark, 2026-11-10):** `coupling_edit` (208 lines)
  drafts one coupling's authored strength; "Writing the candidate is the host's". Its users
  are graphshell-host's `mere_host_coupling`, the one place graphshell-host names the facade,
  and two files in the app's web tree. Offered graphshell-host or incipit. Ruling:
  "graphshell-host (Recommended)", behind `mere-host`. Outcome: applies D58 and D68.

  Links: area The stack · draws on engine-split D58, engine-split D68 · plans engine-split P3 · summary The coupling strength draft lives in graphshell-host behind mere-host.

- **D100. clip is a graphshell-host feature (Mark, 2026-11-10):** `clip` (127 lines) records the
  graph relation for a clip gathered by Fleece; it needs the kernel, incipit and Fleece, and
  nothing calls it. Offered a graphshell-host feature, incipit, or retiring it. Ruling:
  "graphshell-host feature (Recommended)": a `clip` feature beside the goal surface.
  Outcome: applies D58.

  Links: area The stack · draws on engine-split D58, engine-split D91 · plans engine-split P3 · summary Graph insertion for Fleece clips is a clip feature of graphshell-host.

- **D101. routing goes to Turnstone (Mark, 2026-11-10):** `routing` (110 lines) is the
  app-flavoured routing vocabulary over Inker's host-neutral policy; its one code user is
  Turnstone's `standard_route_policy`. Offered Turnstone, graphshell-host or mere-fetch.
  Ruling: "Turnstone (Recommended)". Outcome: applies D58.

  Links: area The stack · draws on engine-split D58 · plans engine-split P3 · summary The app-flavoured route ids live in Turnstone, their one user.

- **D102. Slice 4 is one step (Mark, 2026-11-11):** The slice 4 assessment (Mere `42f5a2f4f`;
  evidence in `Code/testing/engine-split/slice4_assessment.md`) found 41 of Mere's 46 workspace
  crates leave, 452,687 source lines, and 5 stay (`mere-kernel`, `mere-rdf`, `catchment`,
  `pandect`, `mere-facts`, 93,911 lines). No staying crate depends on a leaving one. The move
  order is dependents first, so the reference app (96,423 lines with its sub-crates) leaves
  before or with the toolkit. Offered the app to Turnstone first, everything in one step, or
  the app parked in Graphshell. Ruling: "Everything in one step", over the recommended two
  landings. The toolkit's 33 crates and 13 layers: "One step for the toolkit (Recommended)".
  The app goes to Turnstone and the toolkit to Graphshell in one cutover; the 151 rows in 7
  consumer repos are re-addressed and their 20 lock roots relocked once. Outcome: new; applies
  D23 and D66.

  Links: area The stack · draws on engine-split D23, engine-split D66 · plans engine-split P3 · summary Slice 4 moves the app to Turnstone and the toolkit to Graphshell in one cutover, with consumers re-addressed once.

- **D103. The canvas fold comes first, in Mere (Mark, 2026-11-11):** `pictograph` is still
  71,819 lines; D38's fold of the canvas and signals into `mere-cartography` (5,404 lines) had
  not happened, and iconography D1 sends the faces-only crate to the emblem repository. Offered
  moving pictograph whole and folding later, folding first in Mere, or leaving it in Mere.
  Ruling: "Fold the canvas first, in Mere", over the recommended fold-later. On sequencing
  against the iconography session, which had committed to pictograph 14 times since
  2026-11-09: "it's paused". The fold runs now, before anything moves. Outcome: applies D38.

  Links: area The stack · draws on engine-split D13, engine-split D38 · plans engine-split P3 · summary The canvas and signals fold into mere-cartography in Mere before slice 4's move; pictograph leaves as faces only.

- **D104. The four unassigned crates go by what they are (Mark, 2026-11-11):** The plan named no
  home for `graphshell-door`, `block-adapter`, `graphshell-mere` and `graphshell-dramatis-pane`
  (8,561 lines). Ruling: "By what they are (Recommended)". `graphshell-door` and
  `block-adapter` are libraries below the app and go to Graphshell's `host/`; `graphshell-mere`
  and `graphshell-dramatis-pane` are app pieces and go to Turnstone with the app. Outcome:
  applies D36.

  Links: area The stack · draws on engine-split D36 · plans engine-split P3 · summary graphshell-door and block-adapter go to Graphshell; graphshell-mere and graphshell-dramatis-pane go to Turnstone.

- **D105. The kernel's hest edge is removed (Mark, 2026-11-11):** `mere-kernel` names `hest`
  from Graphshell's contract root. D35 makes that acyclic, but P3's done-condition says "no
  Mere crate depending back". Offered keeping it and rewording the condition, or removing it.
  Ruling: "Remove the edge". The edge is one module, `kernel::actions` (207 lines and a
  39-line test): `ActionId`, "the typed form of `hest`'s catalogue". No Mere crate outside the
  kernel names `ActionId`; six Graphshell files do. The module moves to `hest`, beside the
  catalogue it types, and the kernel drops its `hest` row. Outcome: amends D59, which moved no
  kernel code in P3, by this one module.

  Links: area The stack · amends engine-split D59 · draws on engine-split D35 · plans engine-split P3 · summary ActionId moves from the kernel to hest; mere-kernel names nothing in graphshell.git.

- **D106. Slice 4 moves with history (Mark, 2026-11-11):** Slice 1 imported its five crates with
  history (filter-repo, then a merge of unrelated histories). Offered the same for slice 4's 41
  crates, or a copy at the tip. Ruling: "With history (Recommended)". The size the history
  adds to Graphshell and Turnstone is reported before pushing. Outcome: applies slice 1's
  method.

  Links: area The stack · plans engine-split P3 · summary The toolkit and the app move with their git history, as slice 1 did.

- **D107. Graphshell's design docs are set up in slice 4 (Mark, 2026-11-11):** Graphshell has no
  `design_docs/` and no `DOC_POLICY.md`; Mere's policy moves docs with their code. Offered
  leaving design docs in Mere this slice, or setting up Graphshell's now. Ruling: "Set up
  Graphshell's docs now", over the recommended deferral. Graphshell gets `design_docs/` and the
  canonical `DOC_POLICY.md` core, and the host layer's plans and briefs move with the code.
  Which documents are the host's is sorted as a list and put back before any move. Outcome:
  new.

  Links: area The stack · plans engine-split P3 · summary Graphshell gets design_docs and the canonical DOC_POLICY core in slice 4, and the host layer's plans move with the code.

- **D108. flat_icon stays in pictograph and becomes public (Mark, 2026-11-11):** The fold lane
  measured the dividing line before moving anything. `flat_icon` (365 lines: lowering an icon
  to paint-list commands, and the wash added this week) is private to pictograph and used by
  the canvas, the icon pack and `SymbolImage`. The pack and `SymbolImage` stay, and pictograph
  may not depend on cartography, so it cannot move with the canvas. Offered making it public in
  pictograph, splitting the wash off to cartography, or two copies. Ruling: "Stays in
  pictograph, made public (Recommended)", behind a `paint-list` feature that the pack enables
  and cartography's canvas forwards. `palette_from_rgba` and three backdrop constants the
  canvas reads become public with it. Outcome: refines D38.

  Links: area The stack · draws on engine-split D38, engine-split D103 · plans engine-split P3 · summary flat_icon is a public pictograph module behind a paint-list feature; the canvas in cartography uses it from there.

- **D109. SymbolImage stays with the faces (Mark, 2026-11-11):** `symbols::SymbolImage` (48
  lines) sat behind pictograph's `canvas` feature only for genet-livery's `VectorImage` and
  `flat_icon`; it never uses the canvas, and its one user is the app's web tree. Offered a new
  pictograph feature or moving it with the canvas. Ruling: "Stays in pictograph, new feature
  (Recommended)": a `livery` feature, forwarded by cartography's canvas so the app's manifest
  does not change. Outcome: refines D38.

  Links: area The stack · draws on engine-split D38 · plans engine-split P3 · summary SymbolImage stays in pictograph behind a livery feature.

- **D110. Which design docs move to Graphshell (Mark, 2026-11-11):** The docs sort (evidence in
  `Code/testing/engine-split/slice4_docs_sort.md`, Mere `0c4f551d6`) classed Mere's 227 active
  documents: 39 plainly the host's (11,674 lines), 4 mixed, 50 unsure. Asked which set moves:
  "Rule the 54 uncertain ones first". Ruled in six groups. To Graphshell, each "Graphshell
  (Recommended)": the 9 UI-model designs in `mere_docs/design` (pane UX, gloss navigator, roster
  and frame taxonomy, subgraph derivation, scope model, swatch, node card, one state N windows,
  pool); the 15 resident, door, pairing and route documents (device pairing, family update
  resident, resident inference, app projection offers, native confirmation, resident split, key
  lane, resident host design, remote mere route spec and admission, transport agnosticism,
  delegation petition, session merge-in, interaction model spine, one-node facets layer map);
  and the 10 canvas dynamics and physics documents (node body and face model, dynamics grammar
  plan and brief, graph behaviours, graph view curation, illume text lexer, physics catalog,
  arrangement and dynamics, orrery physics environments, graph projections research). Staying
  in Mere: the four mixed plans, "Stay in Mere whole (Recommended)" (ownership repair, Alembic
  implementation, stack lift, capability door); the stack-wide records, "Stay in Mere as family
  record (Recommended)" (ambiance, GUI catalog, projection scenes brief, Turnstone census,
  application experience brief and plan, MCP-native graph, ranged fetch); and the product and
  workspace research, "Stay in Mere (Recommended)". 73 documents move. Outcome: applies D107.

  Links: area The stack · draws on engine-split D107 · plans engine-split P3 · summary 73 documents move to Graphshell: the 39 host documents, the UI-model designs, the resident and route documents and the canvas dynamics documents; mixed plans, stack-wide records and research stay in Mere.

- **D111. Graphshell's docs get responsa and line-ceiling; the archive stays; the docs move
  before the code (Mark, 2026-11-11):** Three rulings on the mechanics. Tooling: "responsa and
  line-ceiling hooks (Recommended)": Graphshell gets a pre-commit config and its own generated
  `decisions.toml`; the moved decisions leave Mere's manifest and other repos cite them as
  `graphshell:<key>`. History: "Stay in Mere as history (Recommended)": the 139 host-leaning
  archived files stay, cited across repos by path. Timing, offered with the cutover or before
  it: "Now, before the code", over the recommended single repointing. The documents' path
  citations into moved crates are repointed again at U4. Outcome: applies D107.

  Links: area The stack · draws on engine-split D107, engine-split D110 · plans engine-split P3 · summary Graphshell runs responsa and line-ceiling with its own manifest; archived host documents stay in Mere; the documents move ahead of the code.

- **D112. The moved documents are reviewed after the code (Mark, 2026-11-11):** After D110, on the
  73 documents: "I bet some of that is irrelevant now or should be refactored/rewritten" and
  "But, after the code". Asked whether the move waits too, he chose the former of two: move now
  as they are, review after the cutover. The documents moved verbatim (Graphshell `7e34fec0`,
  Mere `dbb505336`); the relevance review and rewrites follow U4. The area root
  `graphshell_docs/` is kept: "Keep graphshell_docs/ (Recommended)". Outcome: applies D110.

  Links: area The stack · draws on engine-split D110 · plans engine-split P3 · summary The 73 documents moved verbatim; their relevance review and rewrites come after the code cutover; graphshell_docs/ stays the area root.

- **D113. What the cutover carries besides code (Mark, 2026-11-11):** Four rulings on U4's edges.
  Hooks: "Carry all of Mere's hooks (Recommended)": clippy, the clippy query hook and cargo
  deny come to Graphshell's two roots with `deny.toml` and the lint tables. App documents:
  "To Turnstone with the app, in U4 (Recommended)" for the three app-only documents. Graphshell's
  README and its `docs/2026-11-06_cut_over.md`: "At the cutover (Recommended)"; the README is
  rewritten against the real tree and the note moves into `design_docs`. The headed check that
  P3's done-condition asks for: "Once, after the cutover (Recommended)", in U6, covering
  slices 2 to 4 together. Outcome: applies D102.

  Links: area The stack · draws on engine-split D102 · plans engine-split P3 · summary Graphshell takes all of Mere's hooks at the cutover; the app's three documents go to Turnstone; Graphshell's README is rewritten then; one headed pass follows in U6.

- **D114. athanor goes to Distillery; Moot's code stays a host port (Mark, 2026-11-11):** Two
  corrections to slice 4's move set. `ports/distillery` is one package, `athanor` (6,076 lines,
  holding the folded `mere-alembic` under D56); the slice 4 assessment classed it as toolkit,
  but the Distillery extraction plan's D20 sends the workshop core to Distillery. Ruling:
  "Distillery, as extraction D20 said". The U4 dry run measures whether that makes a cycle
  between Distillery's root and Graphshell's `host/` root; if it does, that comes back as a
  fork. Moot: the four Moot product documents go to Gemot ("Gemot"); shown that `mere-moot` is
  5,776 lines with 643 touching the toolkit, and that in Gemot it would grow Gemot's build from
  616 to about 1,009 packages and risk a cycle through the resident's site rows, the code:
  "Graphshell, as D55 ruled (Recommended)". Outcome: refines D104; applies D55.

  Links: area The stack · draws on engine-split D55, engine-split D56, engine-split D104 · plans engine-split P3 · summary athanor leaves for the Distillery repository at the cutover; mere-moot stays a host port in Graphshell and only Moot's product documents go to Gemot.

- **D115. D52 waits for Graphshell; three set-asides start now (Mark, 2026-11-11):** D52's merge
  of `block-host` and `authority-host` into `app-host`: "After the cutover, in Graphshell
  (Recommended)". D64, the D53 brief and the move of Distillery's plans: "All three now". D64
  landed in Murm (`6d354536e89`: `murm-ocapn`'s `native` feature); the D53 brief is in
  graphshell.git; Distillery's documents are moving (D116). Also ruled: the app's dead
  `p2panda-store` row is pruned before the move, "Prune p2panda-store only (Recommended)", and
  Cleromancy's two tests that call the kernel's private `facets_mut()` are repointed, "Fix them
  now (Recommended)". Outcome: applies D52, D53, D64.

  Links: area The stack · draws on engine-split D52, engine-split D53, engine-split D64 · plans engine-split P3 · summary D52 is done in Graphshell after the cutover; D64, the uxtree brief and the Distillery document move run beside slice 4.

- **D116. Ten documents move to Distillery, and a repo with rulings runs responsa (Mark,
  2026-11-11):** The Distillery docs sort (`Code/testing/engine-split/distillery_docs_sort.md`)
  found 3 documents plainly Distillery's, 2 mixed and 5 unsure. The three: "Move all three now
  (Recommended)". The extraction plan, with 21 indexed decisions, offered staying, moving or
  archiving: "2, they all should get responsa if they have docs, right?", so it moves whole and
  Distillery adopts responsa and a manifest, cited from Mere as `distillery:<key>`. The six
  remaining uncertain documents: "Move to Distillery". The Alembic implementation plan stays in
  Mere under D110. How far the responsa rule goes: "Every repo that records rulings, after
  slice 4 (Recommended)"; a sweep lists each repo whose docs hold decision headings, with
  counts, before any hook is added. Outcome: new.

  Links: area The stack · plans engine-split P3 · summary Ten Distillery documents, the extraction plan among them, move to distillery.git with responsa; after slice 4 every repo that records rulings gets the hook and a manifest.

- **D117. athanor sits in its own workspace root in Distillery (Mark, 2026-11-11):** The U4 dry run
  (`Code/testing/engine-split/slice4_u4_dryrun.md`; scratch Graphshell, emblem and a five-crate
  Mere all build) found that D114 makes a cycle the checker would fail. Distillery names
  Graphshell's `host/` because athanor's `workshop-native` feature names `cambium-desktop` (2
  use lines); `host/` names Distillery because `mere-resident` names `distillery`, `mere-mesh`
  and `esp`, and `mere-cartography` names `esp`. Offered a nested root, moving `workshop-native`
  out, athanor to Graphshell after all, or an allowlist entry. Ruling: "athanor in its own root
  in Distillery (Recommended)", the device Graphshell uses for `host/`. Distillery's main root
  names no host crate. The dry run confirms how the checker classifies a nested root before the
  cutover relies on it. Outcome: refines D114; applies D35.

  Links: area The stack · draws on engine-split D35, engine-split D114 · plans engine-split P3 · summary athanor is a nested workspace root inside the Distillery repository, so Distillery's main root and Graphshell's host root do not name each other.

- **D118. How much history travels, and emblem's shape (Mark, 2026-11-11):** The import tool cuts
  history at a crate's last rename: with current paths Graphshell gains 1,043 commits (about
  13 MiB) and `mere-resident` loses its months as `ports/djinn`; with the old paths, 1,402
  commits. For pictograph, the whole directory is 297 commits and ten times emblem's size,
  because it holds the deleted canvas; current files only is 46 commits and 121 KiB. Ruling:
  "Old paths for the toolkit; current files for pictograph (Recommended)". The canvas's history
  travels to Graphshell with `mere-cartography`. Emblem: "crates/pictograph, and a tracked lock
  (Recommended)"; emblem starts tracking `Cargo.lock` so its gates run `--locked`. Outcome:
  refines D106.

  Links: area The stack · draws on engine-split D106 · plans engine-split P3 · summary The toolkit moves with history including pre-rename paths; pictograph moves to emblem's crates/pictograph with only its current files' history; emblem tracks a lock.

- **D119. The cutover's loose ends (Mark, 2026-11-11):** Six rulings on the dry run's smaller forks.
  Unowned leftovers in Mere (probes, fixtures that already do not build, the coop relay server,
  retired patch crates, the resident's run scripts): "Follow the nearest user; delete what does
  not build", over the recommended item-by-item round. The four toolkit files over the line
  ceiling with no exception: "Exceptions now, splits after (Recommended)". Path citations in
  the moved documents (107 in 28 Graphshell documents): "Repoint paths at the cutover
  (Recommended)"; a path string is not a rewrite under D112. The backdrop studies read by both
  the canvas and the app: "With the canvas in Graphshell (Recommended)". The resident test that
  checks the app's committed `identity_cards.json`: shown that the file is the resident's own
  projection, "The app builds it live from the resident (Recommended)"; the resident exports
  the builder under `test-fixtures` and the file and its drift check go. Consumers that still
  use pictograph's removed canvas features (Cleromancy 8 lines, Isocosm 3, Knot 13, Woodshed 13,
  Turnstone 149): "In U5, with re-addressing (Recommended)". Taken without a question, as
  having one answer: only used patch rows are carried, consumers relock in strict order with
  Turnstone last, and Graphshell's toolchain file gains `wasm32-wasip2`. Outcome: applies D102,
  D111, D112.

  Links: area The stack · draws on engine-split D102, engine-split D111, engine-split D112 · plans engine-split P3 · summary Leftovers follow their nearest user or are deleted; ceiling exceptions now and splits later; document paths repointed at the cutover; backdrops with the canvas; the app builds the identity cards from the resident; consumers repoint to cartography in U5.

- **D120. The app in Turnstone (Mark, 2026-11-11):** The app dry run (`slice4_u4_dryrun.md`, section
  "App lane") built scratch Turnstone with the app: 297 and 320 lib tests and 585 test attributes
  match Mere. Four rulings. Receipts: the app's history is 1,234 commits; with its receipt
  directories Turnstone grows by 128 MiB, without them by 7.4 MiB, and Turnstone has ruled since
  2026-11-02 that receipts leave the repo: "No; they stay in Mere's history (Recommended)".
  Name: the package is `graphshell` while D23 says the name stops meaning an application:
  "Rename after the cutover (Recommended)"; it moves as `graphshell` and is renamed in Turnstone
  afterwards, the name chosen then. Placement: "Mirror Mere's subpaths (Recommended)",
  `ports/graphshell` and `crates/graphshell/graphshell-mcp`, as rehearsed. Hooks: "The same set
  as Graphshell (Recommended)"; workspace clippy, cargo deny and responsa. Outcome: applies D23,
  D113.

  Links: area The stack · draws on engine-split D23, engine-split D113 · plans engine-split P3 · summary The app moves to Turnstone at ports/graphshell without its receipts, keeps the package name until a later rename, and Turnstone runs the same hooks as Graphshell.

- **D121. Order, fixtures and the last leftovers (Mark, 2026-11-11):** Order: the app lane needed six
  bridge patch rows because Knot and Woodshed still name leaving crates from mere.git, and found
  Inker is an eighth consumer the earlier count missed: "Strict order, no bridge (Recommended)".
  Graphshell, emblem and Distillery are pushed; Mere deletes the moved crates; Inker, Cleromancy,
  Knot, Woodshed, Isocosm, mer3ly and Murm's signalman-desktop are re-addressed and relocked;
  Turnstone goes last and receives the app in that step. Cross-repo fixture reads (the blocks
  test loading block-host's fixture guest by path; Moot's capsule-library scripts reading app
  files): "The owner crate exposes them (Recommended)"; no path points into another repo.
  athanor's history: "With former paths, like the toolkit (Recommended)". The two probes that
  build but have no user, `murm-direct-phy` and `muniment-opfs-probe`: "To their subjects' repos
  (Recommended)", Murm and Eidetic. Outcome: applies D118, D119.

  Links: area The stack · draws on engine-split D118, engine-split D119 · plans engine-split P3 · summary Consumers relock in strict order with Turnstone last and no bridge rows; fixtures are exposed by their owner crates; athanor moves with former-path history; the two unused probes go to Murm and Eidetic.

- **D122. Three forks from the final rehearsal (Mark, 2026-11-11):** The final rehearsal
  (`Code/testing/engine-split/slice4_u4_final.md`) built push-ready branches for Graphshell,
  emblem, Distillery and Mere with every gate green, and raised three forks. `mere-moot` carries
  its own receipts, 94 files and 6.8 MB under `ports/moot/docs/receipts`: "Stay in Mere's history
  (Recommended)", the rule D120 set for the app's receipts; they are left out of Graphshell's
  history as well as its tree. The Turnstone half of Moot's capsule-library check hashes four
  toolkit sources that now live in Graphshell; offered the digests written into the site, cargo
  metadata, a checkout argument, or dropping the hashes: "Graphshell writes the digests into the
  site (Recommended)", so no path points into another repo. Graphshell's cut-over note, archived
  or live: "Live, in technical_architecture (Recommended)". Outcome: applies D113, D120, D121.

  Links: area The stack · draws on engine-split D113, engine-split D120, engine-split D121 · plans engine-split P3 · summary Moot's receipts stay in Mere's history; Graphshell's Moot build writes the toolkit digests into the site for Turnstone to read; Graphshell's cut-over note stays live under technical_architecture.

- **D123. Four forks from the app's first stage in Turnstone (Mark, 2026-11-11):** The cutover is
  pushed (Graphshell `893818f6`, emblem `e577e849`, Distillery `8908f7e4`, Mere `299d19b63`). The
  app lane's first stage (`Code/testing/engine-split/slice5_app.md`) imported the app into scratch
  Turnstone, 612 new commits and about 3 MiB, and raised four forks. Five receipt files are read
  by the app's own code, an example and three tests: "Move to tests/fixtures (Recommended)",
  renamed in the filter so no receipts directory appears in Turnstone's history, as Turnstone's
  own rule of 2026-11-02 did. The Turnstone half of Moot's capsule-library scripts:
  "ports/graphshell/examples/capsule-library (Recommended)". Mere's three crates.io patch rows
  the app's graph uses, `cubek-matmul`, `light-file-dialog` and `peer-facts`: "Carry all three
  (Recommended)", so the app builds against what it was tested with. Nested build directories:
  "target/ at every level (Recommended)". Taken without a question: responsa is in Turnstone's
  hook set as D120 ruled, ahead of D116's sweep (5 decisions found, none live). Outcome: applies
  D119, D120, D122.

  Links: area The stack · draws on engine-split D119, engine-split D120, engine-split D122 · plans engine-split P3 · summary In Turnstone the five code-read receipts become test fixtures, Moot's half sits at ports/graphshell/examples/capsule-library, the three used patch rows are carried, and target/ is ignored at every level.

- **D124. The consumer relock's forks (Mark, 2026-11-11):** The relock lane
  (`Code/testing/engine-split/slice5_u5.md`) built one gated commit each for Murm, Cleromancy,
  Knot, Woodshed, Isocosm and mer3ly; no crates.io version moved in any lock. It found Inker's
  main names nothing from mere.git, so Inker needs no commit, which corrects D121's count of an
  eighth consumer. Four rulings. Graphshell's toolkit needs Genet at `c1d25be4` or later and
  Cleromancy's old Distillery pin doubled chirograph; which revisions the locks take: "The
  revisions Graphshell's host lock records (Recommended)", Genet `3fa3ec62` and Distillery
  `30bf5510`, over each family's main. chirograph 0.0.5's new `icon` field on `AdvertisedAction`,
  at 6 sites in Cleromancy and 2 in Isocosm: "icon: None now (Recommended)"; real icons come
  under the iconography plan. Patch rows already unused before the split (Cleromancy 7, Knot 3,
  and no-op `[patch."mere.git"]` tables): "Remove now", over the recommended wait for U6; a
  second commit per repo, each row proved dead by an unchanged lock, and the same rule is applied
  to Turnstone's 26. Isocosm's main moved three times during the run: "Rebase and push with the
  fast gates (Recommended)"; check on all seven lock roots and the hooks, since the full tests
  passed on the previous base. Outcome: refines D121; applies D119.

  Links: area The stack · draws on engine-split D119, engine-split D121 · plans engine-split P3 · summary Consumers lock Genet and Distillery at the revisions Graphshell's host lock records; new action icons are None for now; patch rows already unused are removed now; Isocosm lands on a rebase with the fast gates; Inker needed no change.

- **D125. The close's document and tooling forks (Mark, 2026-11-11):** U6's first pass
  (`Code/testing/engine-split/slice6_u6_docs.md`) repointed 254 path citations in 40 of Mere's
  documents and removed 35 dead ignore and attribute rows (Mere `ae1ed8ff8`). Four rulings. The
  four uxtree documents in Graphshell hold 25 stale paths, 19 of them written as `mere.git` plus
  a path under "paths as of the audit base": "Only the 6 present-tense paths (Recommended)";
  the 19 stand as the documents' own record. Graphshell's other active documents: 16 citations
  lack `host/`, 125 resolve nowhere there (30 exist in Turnstone, 14 in Mere, 81 nowhere) and 45
  relative links are broken: "Now, the resolvable ones (Recommended)"; the 81 take the policy's
  historical-citation note. Mere's ignore and workspace-exclude rows for `crates/probes/`:
  "Remove it (Recommended)"; the 19 probes are parked in `Code/probes` until sorted. Five
  scripts in Mere that only served crates that left: "Follow the nearest user (Recommended)";
  each moves to the repo whose code it checks if it still runs there and is deleted if not.
  Outcome: applies D112, D119.

  Links: area The stack · draws on engine-split D112, engine-split D119 · plans engine-split P3 · summary In the uxtree documents only present-tense paths are repointed; Graphshell's resolvable citations are fixed now and the rest marked historical; Mere drops its probes convention; scripts that served departed crates follow their nearest user or are deleted.

- **D126. The headed pass's forks, and licences follow content (Mark, 2026-11-11):** The headed
  pass (`Code/testing/engine-split/slice6_u6_headed.md`) built Turnstone `2672b635`, the app's web
  root and Pelt `--locked` and ran them through their self-drive scenarios; a pre-split build
  made the same day (Turnstone `ea971f17`, Mere `325b0083b`) shows the same frames, the web app
  pixel-identical. Three rulings. The resident starting and the native viewer connecting over the
  door has a macOS/Linux driver only: "Run it on the Fedora machine (Recommended)". The four
  frame-receipt scripts went to Turnstone with the app but drive Cambium, now Graphshell's:
  "Move them to Graphshell (Recommended)", which amends D120's placement of them. Five defects
  that reproduce identically before the split (session saving fails on Windows with "Access is
  denied", which is also Turnstone's 48 failing lib tests; the Roster pane runs off the window
  edge; two stale web scenarios; Pelt's `reader` receipt; stale README screenshots): "Record as
  findings, fix later (Recommended)". Separately, on the note that Graphshell has no
  `LICENSES.md` while the rows for third-party content that moved there are still in Mere's:
  "License for sure"; each row follows its content to the repo that now holds it, text verbatim.
  Outcome: refines D120; applies D113, D119.

  Links: area The stack · draws on engine-split D113, engine-split D119, engine-split D120 · plans engine-split P3 · summary The door surface is checked on Fedora; the frame-receipt scripts move to Graphshell; five pre-split defects are recorded as findings; licence rows follow their content to the repo that holds it.

- **D127. The close's second round (Mark, 2026-11-11):** The docs lane's second pass is pushed
  (Graphshell `21c9d70b`: broken relative links 45 to 0, citations resolving nowhere 130 to 5, a
  `LICENSES.md` with 4 rows, `check-physics-engine-off.sh` and the four frame-receipt scripts
  under `host/scripts`; Turnstone `b4961fdb`; Mere `4c91721ef`). Four rulings on what it stopped
  at. Mere's "Ruled 2026-08-27" derivatives paragraph covers Xilem-derived code now in Graphshell
  and tucket and luggage in Eidetic: "Split into per-repo paragraphs", over the recommended
  verbatim copy; each paragraph is drafted beside the original and read by Mark before it is
  committed. `check_port_boundaries.py`, three checks with two homes: "Split it in two
  (Recommended)". `cross-repo-smoke.ps1` and `dr_c_receipts.py`: "Delete both", over the
  recommended move of the smoke to Turnstone. The custody-projection fixture generator, already
  failing in Mere: "Move the whole combined/ directory to Graphshell (Recommended)"; it is run
  once there and committed only if its hashes match what the S79 receipts pin. Taken without a
  question: each pre-split defect is recorded in the Findings of the Turnstone plan that owns
  its surface. Outcome: applies D125, D126.

  Links: area The stack · draws on engine-split D125, engine-split D126 · plans engine-split P3 · summary The derivatives ruling is split into per-repo paragraphs that Mark reads first; the port-boundary check is split between Graphshell and Turnstone; two historical cross-repo scripts are deleted; the custody fixture generator moves to Graphshell if its hashes still match.

- **D128. The derivatives text, the custody fixture and the door build (Mark, 2026-11-11):** Five
  rulings. Graphshell's part of the 2026-08-27 derivatives ruling, drafted by striking words only
  (36 removed, none added): "Yes, as drafted (Recommended)". Eidetic, whose ledger already
  carries its own paraphrase for luggage: "Leave Eidetic as it is (Recommended)". tucket is in
  Murm, which keeps a `THIRD_PARTY_NOTICES.md`: "An entry in Murm's THIRD_PARTY_NOTICES.md
  (Recommended)", the ruling's sentence verbatim. The custody fixture's regenerated manifest and
  lock hashes differ from what the S79 receipt pins, and its sources already differed in Mere:
  "Move it, mark the receipt historical (Recommended)", which settles D127's condition. The door
  check on Fedora built the split cleanly (`--locked`, the first Linux build) but the harness
  starts the resident without a custodian, which the resident has refused since custody isolation
  on 2026-10-26: "Test build with in-process custody (Recommended)", harness unchanged. Outcome:
  applies D126, D127.

  Links: area The stack · draws on engine-split D126, engine-split D127 · plans engine-split P3 · summary Graphshell takes the derivatives paragraph with only words struck; Eidetic is unchanged; tucket's sentence goes into Murm's notices file; the custody fixture moves and its receipt is marked historical; the door surface is checked with an in-process-custody test build.

- **D129. The coop harness is repaired inside U6 (Mark, 2026-11-11):** The second Fedora run
  (`Code/testing/engine-split/slice6_u6_door_fedora.md`) started the in-process-custody resident
  on Linux after the split (sockets listening, status ready, unlocked), then the harness's seeding
  step was refused at the door: "graphshell is not an application this device admits". Since
  admission by consent (2026-11-01) a fresh profile admits no application until the owner
  consents, and the harness cannot answer. `support/graphshell-coop` predates custody isolation
  (2026-10-26), admission by consent and the split's two-repository layout. Offered recording the
  door surface as blocked and closing, repairing the harness inside U6, or pre-seeding consent as
  a second test-only shortcut: "Repair the harness now, inside U6", over the recommended
  record-and-close. The repair is assessed first and its design forks come to Mark before code;
  the split stays open until the door surface passes. Outcome: refines D113, D128.

  Links: area The stack · draws on engine-split D113, engine-split D128 · plans engine-split P3 · summary The coop harness is brought up to custody isolation, admission by consent and the two-repository layout inside U6, forks first, and the split closes only when the door surface passes headed.

## Open questions

- **Closed 2026-11-09 (Mark: "Close all five").** F7 is closed by D19: tinct is in the
  contract root, and merging it into tabard would reverse that. F9 is closed by D12: both crates
  are in Eidetic and both names stay. F18 is closed by D14: the lens registry is deleted, the
  physics presets go with the canvas, and the pack sense stays with the declared-lens plan. F19
  is closed by D26: diagnostics is a host library, and engine stages return typed results as
  data. F20 is closed by D14: scheme routing stays with `mere-fetch`. Also settled this day: F2
  (D42), F8 and F13 (D39), F16 (D40), F21 (D43), F22 (D45), F23 (D47), F24 (D44), P1 F3 (D34),
  P2's forks (D48, D49) and the Eidetic F4 (D50).

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
- **Misfin's home (C).** Settled by D33: it stays in the errand repository, which D32 keeps.
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
- 2026-11-09, P3 slice 1 (branch `es/p3-slice1` in Mere, Graphshell, Eidetic, Turnstone, Knot and
  Isocosm; D35 to D40; receipts in the receipt file): the slice closes. Nothing staying in Mere names
  what left, and the Mere lock only loses packages: 125 removed, none added, no version moved.
  `pelt-core` did not close, because `ports/graphshell` names it (optional, `retained-content`), so it
  stays. The five host crates (34,127 lines) run 296 tests in Graphshell's `host/`, the same count
  and results as in Mere at `e9e9b0aec`, including one failure that already fails there:
  `tabard-workshop`'s `release_accessibility_focus_click_and_numeric_value_keep_separate_authority`.
  Three of the crates read fixtures owned by crates that stay in Mere: Pelt's `examples/`, a Cambium
  example font and a Scenomise map. They were copied with history to the same subpaths under `host/`,
  so no test changed. Under D35 the checker drops `murm/apps/signalman-desktop`'s D17 isolation as
  stale, because that root is now its own node. Counting path rows as well would surface Murm's root
  and `native/` workspaces naming each other by path, so the checker counts git rows only.
- 2026-11-09: D56 landed. `mere-alembic` (3 source files, 682 lines plus lib.rs, README) is folded into `mere-athanor` as `athanor::alembic::{workshop, workshop_surface, workshop_native}`; the crate, its workspace row and its patch row are gone and the name is free. Athanor's existing `workshop` feature now also carries Alembic's records (every consumer already enabled it through `workshop-destination`); `workshop-surface` and `workshop-native` are new athanor features with Alembic's deps. Tests 45 before (41 athanor, 4 alembic), 45 after. The relock removed only `mere-alembic`. D54 is held: `QueryViewSpec::members` calls `mere::subgraph::try_derive_members`, which evaluates SPARQL through `linked-data/query`, so query_view does use SPARQL and cannot leave `mere` behind without `subgraph`'s derivation (fork with Mark).
- 2026-11-09, P4 stage 1 (D3, D9; F1 "Generic over author"): `catchment` exists in `crates/system/catchment` with `pandect::reservoir` (split into `catalog`, `paths`, `store`), `retained_text` (its sealing backend split out as `sealed`) and `shared_root`, 1,099 lines of source and 529 of tests. Pandect depends on catchment and re-exports every old name; `ReservoirStore<B>` and `MereApplicationAccess` are aliases that fix the access author to the journal's `Author`, so stored bytes and every consumer are unchanged. `cargo tree -p catchment` holds no Mere graph or host crate; Eidetic's `chartulary` and `petgraph` arrive through `eidetic` and `sporran`. `catchment/tests/non_graph_domain.rs` opens a `music` mere and a Knot-style operation store through the reservoir. Replication stays in the resident's `personal_sync`. `slot_guard` stays in pandect (F6, open). D47's store half is already `eidetic::image_content`; pandect's `image_store` is the `ImageRef` adapter and the legacy migration.
- 2026-11-09, P4 D60 ids (F4, Mark: "catchment"): `GraphId` and `SessionId` moved from `incipit` to `catchment::ids` with their three tests; incipit re-exports them and so now depends on catchment, so its default feature carries catchment's store dependencies rather than serde and uuid alone. Pandect names the ids from catchment in seven files. Its remaining incipit edge is curation folds only, both through `ViewIntent` and so waiting on F3: `live_view.rs` (`FOLD_RECORD_VERSION` in `LiveViewRecord::validate`, `FoldId` and `FoldRecord` in a test) and `view_intent_store.rs` (`FoldRecord`, and `FoldId`, `FoldStandIn` and `FOLD_RECORD_VERSION` in tests).
- 2026-11-09, D75 gated (Mark: "Gate catchment"): the ids move had made incipit's default resolve 168 packages through catchment. catchment's default is now the ids alone (serde, uuid with v4); the reservoir service and its dependencies (eidetic, muniment, personae, sporran, dirs, async-trait, serde_json, uuid v5) sit behind `store`, which pandect enables. `cargo tree -p incipit -e normal` is 12 packages, the 11 it had before the move plus catchment. incipit packages with catchment's versioned row, but cannot publish until catchment does; catchment cannot publish while the workspace eidetic row enables `product`, which crates.io's eidetic 0.0.3 lacks.
- 2026-11-09, D84: `slot_guard` (170 lines, three tests) moved from pandect to catchment behind `store`; pandect re-exports the module so the four Graphshell port files are unchanged.
- 2026-11-09, P4 D43/D77: `GraphSessionManifest`'s `active_workers`, `engine_profile` and `policy` are one opaque `#[serde(default)] extension` map that pandect round-trips without reading (`extension_get`/`extension_set`). A hand-written `Deserialize` folds the three legacy top-level keys into it under the same names and drops their old defaults (`LEGACY_HOST_KEYS`), so pre-split files read and are written back without top-level host keys. `WorkerKind`, `SessionPolicy` and the session service runner are `mere_resident::session_workers` (keys `active_workers`, `policy`); `engine_profile_store` and `EngineProfileBinding` are `fetch::engine_profile` behind a default `engine-profiles` feature (key `engine_profile`). Pandect keeps `PERSONAS_DIR` and `SESSIONS_DIR` at its root. No consumer outside Mere named any of the moved items.
- 2026-11-09: D54 landed with its fork ruled (Mark: "Move subgraph now"). The premise "uses none" of SPARQL was wrong: `QueryViewSpec::members` calls `subgraph::try_derive_members`, which evaluates SPARQL. `mere::subgraph` (1,783 lines counting its classifier, saved and test files, not the 556 the ruling quoted for `subgraph.rs` alone) and `query_view` (1,619) moved into `mere-view` as `subgraph` (feature `subgraph`, which turns on incipit `arrangement` so the default cone stays free of taffy and petgraph) and `query` (feature `query`, which implies `subgraph` and carries `linked-data/query`; `query-editor` implies it). `mere-view` drops `mere`; the `mere` crate's `query` feature is gone. No consumer outside mere-view and the Graphshell port named either module (Turnstone, Cleromancy, Knot, Woodshed grepped), so no re-export is left in `mere`. The port's `query-views` feature now names `mere-view/query`. Not part of this change: `mere-view`'s `presentation_editor_host` test does not compile under `--all-features` (missing `read_only`), as on origin/main.
- 2026-11-09, P4 D79 (landed before D78, which waited on F7): `ApplicationSettings.theme` is `Option<pandect::StoredTheme>` (id, mode key, `FontStacks` in written order), serialized with the bytes `tabard::ThemeChoice` wrote; two fixtures generated by the pre-D79 store (`pandect/tests/fixtures/application_settings_theme_*.json`, pinned `eol=lf`) round-trip byte for byte. `settings_store`'s legacy migration keeps the mode key as written. pandect no longer depends on tabard. Turnstone converts at its edge.
- 2026-11-09, P4 D44/D78/D86: the session's view log carries `pandect::ViewState { schema, value }`, opaque to pandect, still written in the journal's batch and through `set_view_now`; entries written before the split are a bare object and read as `mere.view-intent/v1`, which is incipit's `VIEW_INTENT_SCHEMA`. `ViewIntent` and its sidecar (`view_intent_store`, 602 lines, its line-ceiling exception retired) are `incipit::view` behind a `view` feature (sceno, hest, serde_json, muniment; kernel as a dev-dependency for the relation-tag test). `live_view` is `graphshell_host::live_view` behind `live-view`. Consumers repointed: mere-overmap, graphshell-mere, the graphshell port's MereHost (which encodes and decodes at its edge), the resident; Cleromancy and Turnstone follow. `cargo tree -p pandect -e normal` names no incipit, tabard or sceno; `hest` remains only through `mere-kernel`'s own edge.
- 2026-11-09, P4 D45/D47 prepared, not moved: `pandect/src/retention/` holds the Eidetic-bound half and names no kernel or pandect type (`mod.rs` 29 lines, `levels.rs` 279: `MemoryRecord`, `MemoryLevel`, `EvictionPolicy`, `evictable_short_term`, `census`; `forgetting.rs` 94: `ForgetProposal`, `apply_forgetting`; `retirement.rs` 134: `RetirementProposal`, `propose_retirement`, `apply_retirement`). `memory_levels` (294) is now the graph adapter that reads nodes into `MemoryRecord`s, and the furnace keeps `propose_forgetting`, image GC and consolidation, re-exporting the moved names. D47's store half already lives in Eidetic as `eidetic::image_content` (save, load, delete, list by digest); pandect's `image_store` is the `ImageRef` adapter, the orphan check is the furnace's, and `migrate_legacy_images` waits for the legacy snapshot adapter. Moving `retention/` into Eidetic is the Eidetic lane's.
- 2026-11-09, P4 D45/D87 moved: `pandect/src/retention/` is `eidetic::memory::{levels, forgetting, retirement}` (Eidetic 8e903bf, `memory` feature = `content_store` + `deleted`, part of `product`, README row added); `eidetic::retention` is untouched. The 8 moved tests run in Eidetic by default and with `--no-default-features --features memory`. pandect keeps `pub use eidetic::memory as retention`, so `pandect::retention::*` is unchanged, and `memory_levels` and `furnace` re-export from Eidetic. Mere's eidetic row already carries `product`. D47's remaining half (`migrate_legacy_images`) still waits for the legacy snapshot adapter.

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
- 2026-11-08: P0 forme lane (branch `p0/forme-incipit`, worktree `worktrees/mere-forme-incipit`, not
  merged). D28 carried out: `mere-forme` (6,692 lines of Rust, tests included) moved into `incipit` as
  `incipit::forme` behind a new `arrangement` feature carrying `serde_json`, `taffy` and `petgraph`
  (forme's never-enabled `petgraph` feature folded in, since every consumer already builds petgraph 0.8
  through the kernel; its `store` feature is now `arrangement-store`); incipit's default stays ids and
  curation on serde and uuid, and it is 0.2.3. forme's kernel dev-dependency is gone (its one use was
  an unused import), so incipit gains no kernel edge, dev or normal; a topology test that had lost its
  `#[test]` attribute runs again. Repathed: the `mere` facade (`mere::forme` re-exports
  `incipit::forme`), `mere-chrome`, `platen` and `pictograph`; no other repository names the crate.
  Both locks change only by the removal and incipit's new edges. Name freed: `mere-forme` (on
  crates.io only once its 0.2.0 publish, `publish = true` in its manifest, is yanked or left).


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
- 2026-11-08: D27's fork F2 carried out, "request and completion pair" (branch
  `p0/clipboard-requests`, worktree `worktrees/mere-clipboard-requests`, not merged). `backstep` 0.2.0
  (still unpublished, still dependency-free) gains `ClipboardRequest { Read, Write(String) }` and
  `ClipboardEvent { Read(Result<String, ClipboardError>), Written(Result<(), ClipboardError>) }`, with
  `ClipboardRequest::fail` (the answer that refuses a request) and `ClipboardRequest::serve` (carry a
  request through a call-shaped `Clipboard`); the sync trait stays. Bound: rootstock's iced embedding
  (iced's `Event::Clipboard` maps to `ClipboardEvent`, answers to other widgets' requests are ignored by
  a pending count, a Paste's text arrives as `SessionInput::Text`, other answers as the new
  `OwnerInput::Clipboard`; Copy and Cut write a selection the host supplies and refuse `Unavailable`
  without one, until Genet's selection query lands) and the Graphshell web share (`web_share::clipboard`
  takes the request inside the gesture and completes with `Written`; the status line now names the
  `ClipboardError` rather than the browser's message). Half-bound: chrome's frame model accepts
  `Written` answers in `FrameHostInput::clipboard_completions` and re-exports the pair, but no host
  port exists in any repository to carry `ClipboardRequest::Write` out, so that leg is a fork. Left:
  rootstock's egui embedding (Copy and Cut wait on the same selection query). Locks gain only the
  `backstep` edges (rootstock, `graphshell-web`).
- 2026-11-08: D29 carried out (branch `p0/frame-model-hest`, worktree `worktrees/mere-frame-model-hest`,
  not merged). `mere-chrome` takes `hest` (already a workspace row; the lock gains only that edge, at
  graphshell `d80316db`). `FrameHostInput::clipboard_completions` and the backstep pair's re-export
  are gone; `FrameHostInput::outcomes: Vec<(String, hest::Outcome)>` carries the host's terminal
  results keyed by invocation id in arrival order, and the new `FrameStep { view_model, invocations }`
  is the runtime's per-frame output, the invocations beside the view model rather than in it. No
  runtime in Mere, Graphshell or Turnstone constructs `FrameViewModel` or consumes `FrameHostInput`
  (`GraphshellRuntime` survives only in doc comments), so `FrameStep` is the declared exit, like the
  input it answers. The new `chrome::effects` declares `clipboard:write_text` (`namespace:name`, as
  hest's catalogue and rootstock's `window:*` ids are written; outside the shared catalogue until a
  second host offers it) under argument schema `chrome.effects/v1`, the text as plain UTF-8 bytes:
  the schema id is the version header, and one string gains nothing from postcard, which chrome does
  not otherwise take. `read_clipboard_write_text` is the host's decode, refusing a foreign id, schema
  or effect. hest's `Invocation::id` is the command id, not a per-call id, so two writes in flight are
  told apart only by issue order. backstep is untouched; no in-repo constructor used the removed field.
- 2026-11-08: rootstock's egui embedding binds Copy and Cut (branch `p0/egui-clipboard`, worktree
  `worktrees/mere-egui-clipboard`, not merged). `Input::clipboard_command` maps egui's `Event::Copy`
  and `Event::Cut`; `Input::clipboard(command, selection, &mut dyn backstep::Clipboard)` writes the
  selection, which the owner reads from the session's `selected_text()` and passes in (the input
  never holds the session; `None` refuses `Unavailable`, as iced does), through `EguiClipboard`, a
  call-shaped `Clipboard` over egui's `CopyText` output command (egui answers no write, so a queued
  write is `Ok`; it serves no reads, its paste already arriving as `Event::Paste`). A Cut whose
  write succeeded also yields an unmodified Delete press and release, which Genet's livery and
  scripted sessions apply to a focused control's non-collapsed range; a document range is copied
  and stays. Left: the Genet relock to fe752369 (`document-session-api` 0.1.2 with
  `selected_text`; bole by `rev = ddf4e31`) is a fork, because webcue 0.1.2 from crates.io pins
  `document-session-api = "=0.1.1"`, the git patch then misses it, and inker's `cue.rs` mixes the
  two copies (seven E0308); until webcue moves, owners have no `selected_text()` to call.
- 2026-11-08: audit of this plan, the doctrine and the cycle-breaks plan against the `main` of all 35 `merely-made`
  repositories (Mere `fc15139f7`; the checker's output is in the cycle-breaks plan, "The P0 result"). Corrected
  in place as stale fact, no decision changed: the Status block (two blocks, one saying nothing had moved);
  `mere-framing` and `state-witness` listed as staying in Mere after D12 moved them; the Seiche crate list
  (`quint-shaders` gone, `seiche-model` missing, one sentence duplicated); Inker's crate count (eleven, since
  `knot-editor-host` retired); `mere-comms`, `chirograph` and the empty `crates/crawl` and `crates/import`
  written as future; the Graphshell leaves that already live in its contract root; the P1 repin marked done; the
  `smolweb` repository name (`errand`); DOC_README's family table rows for Inker, Nematic, Fleece, Mesh, Intel
  and the persona picker. Contradictions that need a ruling were reported to Mark with evidence, not edited.

- 2026-11-08: D30's retirement carried out (branch `p0/chrome-retire`, worktree
  `worktrees/mere-chrome-retire`, not merged; `df0141622`). `frame_model`, `effects` and `host_intent`
  are deleted (1,023 lines); chrome drops `hest` and `incipit`, and the lock only those two edges. The
  audit of what remains is in the receipts: no module has two consumers; `routing` has one
  (ux-events, `ToolSurfaceReturnTarget` alone, not `toolbar` as D30's evidence says); Turnstone's
  `nav::History` is backstep's; `authorities`, `omnibar`, `suggest`, `toolbar` and the rest of `nav`
  have none, Turnstone carrying its own omnibar and address normalising. `mere-chrome` 0.0.1 is not on
  crates.io. Dispositions are pending Mark's ruling.
- 2026-11-08: D31 carried out (branch `p0/chrome-dissolve`, worktree `worktrees/mere-chrome-dissolve`,
  not merged). `routing` (`ToolSurfaceReturnTarget` and its three tests) moved into `ux-events` as
  `ux_events::routing` with history; `crates/shell/chrome` *(historical citation)* <!-- doc-audit: historical-path -->
  is deleted with `authorities`, `omnibar`, `suggest`, `toolbar` and `nav` (`History` was backstep's
  re-export; the classifier had no importer). The member and workspace rows go; the lock loses
  `mere-chrome` and ux-events' edge to it, and ux-events gains `serde_json` as a dev-dependency for the
  moved tests. Doc comments naming the retired frame-model types in the kernel and ux-events are
  reworded. `mere-chrome` is freed. Turnstone still names `mere-chrome` for `chrome::nav::History`
  at four sites; its lock pins an older Mere, so nothing breaks until a Turnstone lane swaps them for
  `backstep::nav::History`.

- 2026-11-11: U4, the cutover (D102, D104, D106, D113, D114, D117 to D121), taken from mere.git
  `325b0083b`. Left, with history: the host toolkit's 34 crates (`graphshell-door` and
  `block-adapter` among them) and their non-members for Graphshell's `host/` at the same subpaths
  (graphshell.git `893818f6c` on base `53584e76`; 2,047 commits with former paths, without
  `mere-moot`'s receipts, which stay in this history under D122); `pictograph` for emblem's
  `crates/pictograph` (emblem.git `e577e8498` on base `4630fab`; its current files' 46 commits;
  emblem tracks its lock); `athanor` for Distillery's nested root `crates/athanor`
  (distillery.git `8908f7e4a` on base `a3041eb`; 16 commits with former paths). The reference
  app (`ports/graphshell`, `graphshell-mcp`), its four frame-receipt scripts and
  `support/{graphshell-coop, browser-history-intake, instrument, link-model}` leave for
  Turnstone, which imports them from `325b0083b` when it relocks last (D121). Deleted under D119:
  `crates/probes/{pack-distribution, wing-three-paths}`,
  `support/fixtures/custody-projection/{host, offer, projection, provider}`,
  `support/family-promotion/native-id-consumer` and `support/patches/{burn-cubecl,
  cubecl-runtime, cubecl-wgpu}`. To their subjects' repositories (D121), copied at
  `325b0083b`: `crates/probes/murm-direct-phy` for Murm's `testing/`,
  `ports/muniment-opfs-probe` for Eidetic's `crates/`. The backdrop study's records stay in
  this history (D120); its ten JSON files went with the canvas (D119). The workspace keeps the
  five crates; the lock went from 1,664 to 651 packages, removals only; 133 line-ceiling
  exceptions left with their files, the query clippy hook checks `mere-rdf` alone, and the
  arXiv corpus rule left `.gitattributes`. Staying documents' path citations into the crates
  that left are U6's.
- 2026-11-11: U5, the consumers (D121, D123, D124), all pushed as fast-forwards. One relock
  commit each, no crates.io version moved in any lock, no bridge rows: Murm `fba8d83a` (6 rows,
  signalman-desktop), Cleromancy `28801b73` (25 rows, 8 canvas lines to `mere-cartography`),
  Knot `165dae81` (20 rows, 13 lines), Woodshed `8b726b69` (31 rows in 4 lock roots, 13 lines),
  mer3ly `7952bad5` (18 rows), Isocosm `50accf72` (32 rows in 7 lock roots, 3 lines; rebased
  onto a moving main and landed on the fast gates under D124). Inker's main names nothing from
  mere.git and took no commit. Genet and Distillery are locked at the revisions Graphshell's
  host lock records. Turnstone went last, `2672b635` on base `fd57dd3f`: the reference app at
  `ports/graphshell` and `crates/graphshell/graphshell-mcp` with 612 new commits and no receipts
  directory in its history, the five code-read receipts as test fixtures, Moot's half at
  `ports/graphshell/examples/capsule-library`, three documents, the support harnesses, the three
  carried patch rows, and Graphshell's hook set; 297 and 320 lib tests and 585 test attributes
  match the dry run, and its lock takes only the five engine crates from mere.git. A second
  commit per repo removed patch rows that were already unused, each proved by an unchanged lock
  (Cleromancy `c7ca8ed0`, Knot `ef69aef0`, Woodshed `f96b878a`, Murm `111bdf9d`, mer3ly
  `f5d6f06e`, Isocosm `413489b9`; Turnstone's 26 in `2672b635`). The family cycle checker, run
  with `--remote` over 35 repositories and 94 lock roots at these heads, exits 0; it warns of a
  resolved cycle inside three locks (Cleromancy, Graphshell `host/`, Turnstone) where athanor and
  the resident meet in one graph. Failing identically before and after, not fixed: Turnstone's
  48 Windows lib tests, Knot's 18, Cleromancy's `granted_ambient` and `authority_corpus`,
  Isocosm's `family_network`, mer3ly's favicon test. Not verified: anything headed, Linux or
  macOS, Mesocosm's long tests on the final base. Evidence: `Code/testing/engine-split/`
  `slice4_u4_final.md`, `slice5_u5.md`, `slice5_app.md`. U6 remains.
