# Stack seams plan: catalog, shared device, actions, determinism, two words

**Date:** 2026-10-04
**Status (2026-10-06):** in progress. Seventy-seven rulings in twenty-six rounds
(S1 to S77); P1 landed on main (`1633be0c`); P2 staged (S27 to S31, four
stages in §3.1) and landed on main (`40d7ae5e`); S32 (F19) landed on main
(`48c08dee`); the S14 pass (S33) done (`8d7702d2`, receipt
`support/doc-audit/d2/remediation_2026-10-06.md`); S66's code follow-on
(S70 to S75, the smolweb fidelity plan's WS4) landed bar R6 (`b4f14f2c`); P3
and S7 done as documents; S3 to S6 carried into the dynamics grammar plan (G8, G9); S9 done by the identity
lane (`b52edea7`).

A note sent to Mark listed weak seams in the stack. Each claim was checked
against the code (§1); six seams proved real and went to him as
multiple-choice rounds (§2). His answers are quoted verbatim; anything beyond
them is marked *Reading, not ruled*. The same session ruled the
[graph semantics plan](2026-10-04_graph_semantics_plan.md).

## 1. Findings

Verified 2026-10-04 against Mere `c34449bd`.

- **F1. The projection bridge exists, but it is one port's and covers two
  families.** The note said scenograph has no consumers and no bridge to
  `Score`; that is stale. Graphshell re-exports scenograph's types
  (`ports/graphshell/src/projection_editor.rs` 18), and its
  `projection_compile` has emitted a `sceno::Score` since `534ae1c6`
  (2026-09-06). `arrangement_for` (`projection_compile.rs` 666-682) maps two
  ids by hand: `grid.default` to `Grid` with a 184 by 84 cell and 8 columns
  written in, and `scatter.default` to `Geographic`; every other id returns
  `None`. Scenograph's `Arrangement.kind` is "a registered arrangement id
  resolved by the host catalog" (`crates/cambium/scenes/scenograph/src/lib.rs`
  147-160), its crate doc says hosts own "catalog resolution ... compilation",
  and scenomise already keeps a `SolverCapability` registry keyed by
  `ArrangementId` (`crates/cambium/scenes/scenomise/src/registry.rs` 50-90).
- **F2. Three `Arrangement` types, two senses.** Scene placement:
  `sceno::Arrangement`, a closed enum of analytic families plus `Custom`
  (`crates/cambium/scenes/sceno/src/score.rs` 246), and scenograph's authoring
  request above. The workbench: `forme::Arrangement`, "a small graph of nodes +
  relations ... Geometry-free" (`crates/forme/forme/src/arrangement.rs` 129),
  with the kernel's Arrangement relation family `FrameMember | TileGroup |
  SplitPair` (`edge_taxonomy.rs` 96-100). The dynamics plan's F11 rules that an
  arrangement is positions, not motion.
- **F3. Actions have a model and a ruling, not yet joined.** Chirograph's
  `AdvertisedAction { intent, label, explanation, payload_schema, .. }` is "an
  action carried into accessibility and permission surfaces", classed by
  `IntentEffect` as `Curation`, `DomainTruth` or `ExternalEffect`
  (`crates/chirograph/src/lib.rs` 105-130). The dynamics plan's F21 rules that
  one domain binding supplies "the actions both honour (drag, pin)"; its
  binding phase names channel resolution and the affinity toggle, not actions
  (dynamics plan, line 263).
- **F4. Force laws declare no determinism, and seiche does not repeat.**
  `SolverCapability.is_deterministic` exists; seiche's `Term` declares topology,
  kernel, state, currency, class, metric and observable only
  (`crates/conatus/seiche/src/terms.rs` 39-50); the dynamics plan does not
  mention determinism. Measured with two probes kept with their source and logs
  under `Code/testing/mere/`:
  - `rapier-determinism/`: rapier2d 0.33 with and without
    `enhanced-determinism` on a seiche-shaped world (balls, no gravity, linear
    damping, a mass-scaled centering force, dt 1/60), with a control asserting
    active contacts. At 500, 2,000 and 5,000 bodies (930, 3,957 and 12,257
    contacts): 0.345 and 0.343 ms per step against 0.346 and 0.338; 1.83 and
    1.97 against 2.08 and 1.98; 9.58 and 9.23 against 9.41 and 9.51. The
    difference sits inside the run-to-run spread. Each build repeated bit for
    bit; the two builds' fingerprints differ, so the feature changed the math.
    A first run with zero contacts measured nothing and was discarded.
  - `seiche-repeat/`: a 300-node graph with `BarnesHutRepulsion`, `EdgeSpring`
    and `NodeExclusion`, 300 ticks, positions hashed in key order, run as five
    processes: five different fingerprints (outermost radius 352.77 to
    354.20). Force paths sum over `bodies_by_node`, a std `HashMap` whose order
    is randomized per process (`barnes_hut.rs` 114, `forces.rs` 235,
    `laws/hold.rs` 28); a search for clocks, RNGs and parallel iteration in the
    step found none besides timing code in `tensor_forces.rs`.
  - Seiche's own laws call std transcendental functions (`powf`, `exp`, `sin`
    and others in `laws/kuramoto.rs`, `laws/boids.rs`, `laws/linlog.rs`,
    `laws/anneal.rs`), which `enhanced-determinism` does not cover.
- **F5. "Scene" has four senses.** `sceno::Scene`, the placed result a
  projection produces (`sceno/src/scene.rs` 182); the scene recipe that makes
  it (`SavedSceneV1`); seiche's `SceneSpec`, a physics world of bodies,
  joints, fields and emitters (`seiche/src/scene_spec.rs` 234); and netrender's
  `Scene`, the paint op list, in a sibling repository. Conatus already calls
  its body runtime a world (`BodyWorld`, `crates/conatus/conatus/src/world.rs`
  167).
- **F6. Cambium's desktop host makes its own device.** The note's claim holds.
  `cambium-genet-winit-host` owns the event loop (`lib.rs` 514), boots through
  `SurfaceHost::boot_with_transparency` (383) and never calls genet's
  `SurfaceHost::from_shared_core` (`genet/components/genet-winit-host/src/lib.rs`
  61, "without allocating a second wgpu device"), and boots a fresh surface
  and core on every resume (636-650). Its doc rules out multi-window "per the
  Signalman desktop scope (retinue, 2026-08-09)" (18-19). The ratified GPU
  regime puts tenants on one device and one queue
  ([spatial compute plan](../../archive_docs/2026-10-06_completed_plans/2026-08-13_spatial_compute_plan.md),
  lines 25 and 56). Consumers: mere-view, cambium-rootstock,
  cambium-genet-web-host and pelt's desktop port here; woodshed, hocket and
  redshank in the woodshed repository.
- **F7. G2 has code.** The note said G2 has its rulings and no code; branch
  `grammar-g2` holds 50 changed files, 5,042 insertions and 99 deletions
  against main, unmerged.

A second pass, the same day at `d2e5c273`, censused public type names defined
in two or more crates.

- **F8. Most duplicate names are layering, not seams.** `Graph` (chartulary's
  generic graph under the kernel's wrapper), `NodeKey` (one petgraph
  `NodeIndex` alias in four crates; `inker/src/routing/ids.rs` 20 says so),
  `Term` (a force term in seiche, an RDF term in chartulary), `Admission`
  (storage in stickleback, session in murm, observation in apparatus),
  `Selection` (chirograph's portable noun beside a practice-workspace enum in
  graphshell), `ProjectionRequest` (cartography's in-process call beside
  chirograph's wire request), and the three tile structures (forme, platen's
  `TileLayout`, cambium's `TileTree`), which the
  [workbench component plan](../../archive_docs/2026-10-06_completed_plans/2026-08-31_workbench_component_plan.md)
  rules as one pipeline: "Mere's Forme remains durable graph-arrangement
  authority; Platen compiles a Forme arrangement into a Workbench
  presentation". document-host's script `Grant` derives from servitor's
  authority (`Grant::from_authority`, its README), so it is not a second
  sandbox. One stale comment: platen's `Workbench` alias says the name "will be
  reused by the future projection-authoring component"
  (`crates/platen/platen/src/workbench.rs` 410-415); projection authoring is
  scenograph, and cambium's `Workbench` holds the name.
- **F9. Two `ViewIntent`s that never meet.** cartography's is a per-call
  request, "what the user is trying to see right now" (target size, focus,
  filter; `crates/canvas/cartography/src/request.rs` 42). pandect's is the
  persisted per-session view record: hidden relations, folds, camera
  (`crates/system/pandect/src/view_intent_store.rs` 118), which is the swatch
  design's curation plane.
- **F10. Two `PersonaId`s, an id and a key.** personae's is a UUID
  (`crates/dramatis/personae/src/lib.rs` 120); mien's is "a persona's leaf
  identity (its public key)" that "derives in production from master +
  persona_id" (`crates/moot/mien/src/persona_chain.rs` 35).
- **F11. The README and TERMINOLOGY behind the record.** The README called Mere
  "the library behind a graph-first browser", where TERMINOLOGY (ruled
  2026-09-05) says Mere is the platform and "Not a browser"; it counted 92
  crates (the workspace has 100 members, 87 crates and 13 packages under
  `ports/`) and four ports (nine port directories are members); its status was
  dated 2026-08-12. TERMINOLOGY had no pandect entry while crediting Eidetic
  with saving sessions; pandect ("everything a Mere session gathers under one
  cover") holds them and depends on eidetic and muniment, as the ambiance
  design's §5 describes.
- **F12 (2026-10-05, P1's assessment). The registry is the custom catalog
  only.** F1 and S1's options called scenomise's `SolverRegistry` the
  arrangement catalog. It holds only solvers reached through
  `sceno::Arrangement::Custom`: "The eleven named families never touch the
  registry" (`scenomise/src/registry.rs`, `solve_via`), and nothing registers
  them, so the named families have no ids. Ids already disagree: scenograph's
  default `kind` is `"grid"` (`scenograph/src/lib.rs`, `Arrangement::default`)
  while graphshell uses `"grid.default"` and `"scatter.default"`. Authoring
  offers a string direction, an integer spacing and string options, where
  sceno's families take typed parameters.
- **F14 (2026-10-05, P1 as built).** Stack reads its layer from
  `ScoreItem::axis`, not `ScoreItem::layer` (sceno's `Stack` doc), so S21's
  "integer x as its layer" is a numeric axis held to whole values. Today's
  scatter compiled with `invert_y: false` where sceno's `Geographic` default
  is `true`; the catalog keeps `false` so saved scatter recipes do not flip.
  graphshell lays cards out at 164 by 68 scene units and scales the whole scene
  at paint (`web_practice.rs`, `card_size`), so that scene-unit card, not the
  zoomed one, is what it supplies. The new scenograph-to-sceno edge touches two
  lockfiles (the root and `ports/distillery/probe/remote-fixture`), one line
  each; graphshell's web workspace keeps a gitignored local lock. During the
  gates another session cleared the shared `C:/t/cargo-targets/mere` twice
  mid-build, so P1's gates ran in an isolated target.
- **F13 (2026-10-05). The compiler moved under another lane.** At `c79bb8c2`
  (00:52), for its relationship-recipe pass, the projection grammar lane moved
  graphshell's compiler into `scenomise::projection` (1,147 lines) and made
  scenomise depend on scenograph; its plan names its integration worktree "the
  single owner of shared compiler changes for this pass", with consumer
  qualification in Knot and Woodshed in progress. Moved verbatim, and so still
  open for P1: the dataset types in scenomise (`projection.rs` 525-570), the two
  string ids (508-509), and `arrangement_for` with the 184 by 84 cell and 8
  columns (1099-1125). At 14:38 that lane was idle and no branch held unmerged
  compiler edits.
  **Corrected 2026-10-05:** the attribution above is wrong. Projection grammar
  [78751e] reports it wrote none of `c79bb8c2`, `b2f67356` or `4a2560ed`, nor
  the plan text naming the integration worktree; the commits carry only
  `mark-ik`, and Mark knows the session that did. That pass's next step is
  Woodshed-side ("The Woodshed checkout lane owns implementation"), consuming
  the compiler through a pinned mere revision. Measured for P1's effect on it:
  grid pitch is `cell + gap` (`scenomise/src/solve.rs` 312-322), so today's
  184 by 84 cell at spacing 16 gives a 200 by 100 pitch, and a cell fitted to
  the 164 by 68 card (the footprint written in at `projection.rs` 697) gives
  180 by 84; `columns` is unused there, since the grid ranker gives every item
  an explicit cell.
- **F15 (2026-10-05, reported by the Knot lane after P1 reached origin at
  `19e6dc9f`). P1 breaks Knot.** P1 made `compile_relationship_snapshot` and
  its `_with_limits` form methods on `ProjectionCompiler`, built from
  `ItemSizes`, which has no default on purpose (S20). Knot `54bb8cd` still calls
  the free function: `crates/knot-composition/src/retention.rs` 39, a test
  support file, and `apps/desktop/src/composition/recipe.rs` (checked here:
  eight calls in that file). Mere builds today only because its
  `[patch."…mere.git"]` table serves Knot no `scenomise`; ruling 586's row,
  added with djinn's Knot repin, makes `knot-composition` fail with E0425.
  P1's progress entry named Woodshed's consumers as not run and missed Knot,
  which compiles inside mere's graph through djinn. Mark ruled that the Knot
  lane adapts Knot (repin onto `19e6dc9f`, the three sites onto
  `ProjectionCompiler::new(ItemSizes { .. })`, Knot's card size as a fork to
  him), recorded in the burn plan's §13.46
  ([burn plan](2026-08-09_burn_0_22_migration_plan.md)) at `ac7f906d`. P2
  does not touch `ProjectionCompiler` or `ItemSizes`.
  **Corrected 2026-10-05:** the counts above are wrong. Knot `54bb8cd` has
  twelve call sites: one in `retention.rs` and eleven in
  `apps/desktop/src/composition/recipe.rs` (five in code, six in its tests; the
  twelfth mention there is the import), and `tests/support/recipe.rs` makes
  none; "eight" came from output cut off at ten lines. The Knot lane reports
  Mark's ruling on Knot's card, "Measured from Knot's font": the desktop
  measures each recipe's widest occurrence label in its own font plus button
  padding and hands that to the `ProjectionCompiler`, with retention
  validation's nominal size, the measuring path and the padding to come back
  to him as forks; recorded in the burn plan's §13.46 ("Knot's card size under
  P1", `7d6dc003`; how it measures, `102aa548`). That lane also notes P1 moves Knot's spacing whatever card
  is chosen (card plus gap, where a fixed 184 by 84 cell was), and that Knot is
  the first host to measure its card rather than declare it.
- **F16 (2026-10-05, P2 stage 1's control). wgpu's device equality cannot tell
  two boots apart.** wgpu 30 compares a `Device` by a per-instance id
  (`impl_eq_ord_hash_proxy!(CoreDevice => .id)`), and each `RenderCore::boot`
  makes its own instance, so devices from two boots compare equal. Stage 1's
  control first passed wrongly on exactly this; the receipt now compares device
  identity. The producer registry detects a changed device with the same `!=`
  (`cambium-rootstock/src/producer/registry.rs`, `prepare`), so under the old
  reboot on every resume a new device went unnoticed there. After stage 1 a
  host's device never changes, so nothing reaches it today; recovery from a
  lost device would need the check to compare identity.
- **F17 (2026-10-05, for P2 stage 4). Several surfaces through one core must
  key their rasterization.** Genet's `RenderCore::rasterize_for` doc: a host
  that rasterizes several surfaces through one core must key each, or every
  tile is dirty on every frame (234 of 234 in the shell paint plan's
  measurement). Rootstock's redraw calls the unkeyed `rasterize_scaled`
  (`frame.rs`), harmless with one window; stage 4 keys it per window, and
  `presentation_host` is already unique per host.
- **F18 (2026-10-05, P2 stage 4's control). On Windows, one animating window
  starves another.** Winit 0.30 asks for a frame with
  `RedrawWindow(RDW_INTERNALPAINT)` and delivers it on `WM_PAINT`
  (`platform_impl/windows/window.rs` 152, `event_loop.rs` 1276), and Win32
  hands `WM_PAINT` to the first window needing paint every time, so a window
  that asks for its next frame from each frame, as an animating one does,
  keeps being chosen. Measured: with both windows animating, B presented 4,981
  frames from 4,979 redraw events while A had one event and presented nothing
  more, until the driver gave up. Deferring the request to the idle turn did
  not change it (the request still precedes the next message wait), and
  forcing the paint from inside a callback (`RDW_UPDATENOW`) only re-queues it,
  since winit's `WM_PAINT` handler buffers while a callback runs. The
  multi-window entry therefore notes each window's requests and, on Windows
  only, draws each window that asked in the idle turn, through the same
  handler a delivered paint runs; `Fifo` presentation paces each to its
  monitor, and paints the OS sends still arrive as before. Elsewhere it hands
  the requests to the platform at the idle turn, so X11, Wayland and macOS keep
  their own frame delivery. After: A presented 8 frames while B presented 9.
  *Reading, not ruled*: Windows-only because that is where the starvation was
  measured; drawing from the idle turn everywhere would bypass Wayland's frame
  callbacks, under which a hidden surface's `Fifo` acquire can block, and
  piggybacking one window's frame on another's paint has the same hazard.
  Two animating windows on X11, Wayland and macOS are unmeasured.
- **F19 (2026-10-05, reported by the Knot lane). P1 accepts degenerate card
  sizes.** Run through five relationship recipes at seven `ItemSizes` cards,
  among them 0 by 0, NaN and 1e6, scenomise gave byte-identical outcomes and
  issue lists, with no complaint about 0 by 0 or NaN. Since S20 has hosts supply
  the size, a host bug that hands over 0 or NaN passes silently. How to refuse
  them (a `Result` from `ProjectionCompiler::new`, a typed compile issue, a
  debug assertion) is a fork for Mark; the first changes the signature the Knot
  lane is adapting to, which was told to proceed on the current one.
  **Ruled and built 2026-10-05:** a typed compile issue (S32), landed on main
  at `48c08dee`.
- **F20 (2026-10-05, P2 stage 4's review). What the multi-window pacing leaves
  unmeasured.** Serving each window that asked in one idle turn, each acquire
  waiting on its swapchain, couples windows on monitors of different refresh
  rates to the slower (a 60 Hz and a 144 Hz window would both draw at 60);
  S26's "at its own monitor's rate" holds for windows on like monitors. Whether
  Win32's modal move and size loops still run the idle turn (so an animating
  window keeps drawing while another is dragged) is unmeasured, as is whether an
  occluded but not minimized window's present blocks. A panic inside a turn
  leaves the lent runner in that window; nothing catches it today.
- **F21 (2026-10-05, S32's gates). A graphshell carrier test times out under
  load.** `carrier::tests::p2panda_murm_grant_is_refused_before_projection_bytes`
  waits 10 seconds for the server to accept a projection session. Run within
  graphshell's `projection` test filter (35 tests) it timed out at that accept
  in 2 of 5 runs with S32 and in none of 4 without it, and alone it passed 3 of
  3 in about 2.5 seconds each. It reads no scenomise code, and both failures
  came while other builds and subagents were loading the machine. *Reading,
  not ruled*: a timing flake in a fixed timeout, not caused by S32; it is
  recorded here and not fixed, as graphshell's carrier is outside this plan.

## 2. Rulings

Mark's answers, from multiple-choice rounds; each is the option label quoted
verbatim unless marked as his free text.

**Ruling S1.** *Where are arrangement ids resolved?* Options: resolve through
scenomise; a shared catalog with host extras; hosts keep their catalogs.
Mark: **"Resolve through scenomise (Recommended)"**. Follows: scenomise's
registry is the catalog. Scenograph ids resolve there into `sceno::Arrangement`,
with parameters from the definition, in one shared compile step every host
calls; graphshell's hand map and its written-in sizes go, and every registered
family becomes authorable. Amends scenograph's "hosts own catalog resolution
... compilation".

**Ruling S2.** *Should Cambium's desktop host share one GPU device?* Options:
boot one core and share it, still one window; a shared core plus multi-window;
wait for a tenant. Mark: **"Shared core plus multi-window"**. Follows: the host
boots one `RenderCore`, makes each window's surface with `from_shared_core`,
hands the core to tenants (conatus, Burn), keeps it across suspend and resume,
and lifts the single-window limit, every window on the one core. Amends the
Signalman desktop scope's "no multi-window" for this host; that scope's own
record is retinue's.

**Ruling S3.** *What is a permitted action?* Options: one model,
`AdvertisedAction`; separate gesture permissions. Mark: **"One model:
AdvertisedAction (Recommended)"**. Follows: a domain binding's permitted
actions are chirograph `AdvertisedAction`s; drag and pin are `Curation`-effect
actions, since positions are not graph truth; one path into accessibility and
permission surfaces. The dynamics lane carries this into F21's binding.

**Ruling S4.** *Should force terms declare determinism?* Options: per term,
same build; per term, cross-platform; no declaration. Mark (free text): **"How
large is the performance cost of enhanced determinism?"** Answered by the two
probes in F4. Put back with options: ordered, declared, cross-platform;
ordered, declared, same build; declare only. Mark: **"Ordered, declared,
cross-platform (Recommended)"**. Follows: seiche iterates bodies in key order
so a build repeats; each `Term` declares deterministic, seeded or
nondeterministic, as a solver does; rapier's `enhanced-determinism` goes on;
seiche's own transcendental math routes through `libm`, with its cost measured
as part of the work. Runs repeat across platforms. The dynamics lane carries
this.

**Ruling S5.** *Which sense keeps "arrangement"?* Options: the workbench sense
becomes forme; the workbench sense becomes tiling; keep both and
disambiguate. Mark: **"Workbench sense becomes forme (Recommended)"**.
Follows: a workbench arrangement is a **forme**, the printing word the crate
already carries; "arrangement" means positions only, as F11 rules.
`forme::Arrangement` and the kernel's Arrangement relation family rename in
code opportunistically, under TERMINOLOGY's `edge` rule, with the persisted
family tag still read.

**Ruling S6.** *Which sense keeps "scene"?* Options: scene stays projection and
physics says world; keep all and disambiguate. Mark: **"Scene stays
projection; physics says world (Recommended)"**. Follows: "scene" keeps
sceno's sense and its recipe; seiche's `SceneSpec` is a world spec, matching
conatus's `BodyWorld`; netrender's op list is "the paint list" in prose, its
type being another repository's. Code renames opportunistically.

Round 2, from the second pass (F8 to F11).

**Ruling S7.** *What happens to the README?* Options: fix facts and add the
ruled words; rewrite from the record; wait for PROJECT_DESCRIPTION. Mark:
**"Fix facts, add ruled words (Recommended)"**. Follows: the opening follows
the 2026-09-05 ruling (Mere is a platform, Turnstone the browser); counts,
ports and the status date are corrected; the graph's words (resource, node,
strata, aspect) and forme and world join Design vocabulary in a few lines; the
rest of the prose stays. PROJECT_DESCRIPTION.md remains Mark's to found.

**Ruling S8.** *Which `ViewIntent` keeps the name?* Options: pandect's becomes
curation; cartography's becomes a request; keep both. Mark: **"pandect's
becomes curation (Recommended)"**. Follows: pandect's stored record is the
curation record, named for the plane it holds; cartography's per-call
`ViewIntent` keeps its name. Renamed when touched, with the stored file name
still read.

**Ruling S9.** *What is mien's `PersonaId`?* Options: it becomes
`PersonaKey`; keep both. Mark: **"mien's becomes PersonaKey (Recommended)"**.
Follows: the key is named a key; personae's id keeps `PersonaId`. The identity
lane renames it.

**Ruling S10.** *Who holds a mere's sessions?* Options: pandect holds and
Eidetic archives; pandect is part of Eidetic; leave TERMINOLOGY as is. Mark:
**"Pandect is part of Eidetic"**. Follows: Eidetic names the memory family
(muniment's stores and journals, pandect's session record, the codicil
archive), and pandect is its session layer; TERMINOLOGY gains pandect and
amends Eidetic. *Reading, not ruled*: the crates stay where they are; whether
pandect moves under `crates/eidetic/` is not ruled.

Round 3, 2026-10-05, on carrying out P1 and P2.

**Ruling S11.** *Who builds P1?* Options: this session, in a worktree; a brief
for another lane; park it. Mark: **"Build here, in a worktree
(Recommended)"**. Follows: this session builds P1 on its own branch under
`Code/worktrees/`, stops at forks, and nothing reaches main until Mark reviews
it.

**Ruling S12.** *How much of the compile does P1 share?* Options: the whole
compile over a trait; catalog resolution only; a portable dataset contract.
Mark (free text): **"Wait, why didn't we want a portable contract…?"**
Answered: graphshell's comment ("supplied to the compiler without a product
dependency or a portable product-data contract",
`projection_compile.rs` 135) applies the
[remote projection host plan](../../archive_docs/2026-10-06_completed_plans/2026-07-22_graphshell_remote_projection_host_plan.md)'s
charter, ruled with Mark on 2026-07-22: portable contracts carry placement and
representation only and stay product-free (line 81), "Keep product adapters
beside their source truth" (579), and "later radio facts add data without
widening the portable contract" (389). The dataset itself is generic (named
typed fields, values, occurrences with source references), already
serializable, and loaded as JSON only inside one host (a fixture and the
editor preview, `web_projection.rs` 65); it never travels the remote protocol,
where endpoints compile locally and send scores. Put back with options: a
shared generic table, local; a trait over host data; catalog resolution only;
a portable wire contract. Mark: **"Shared generic table, local
(Recommended)"**. Follows: the generic dataset moves to scenograph as a
host-neutral type, documented as the in-host input to compilation and never
part of the remote protocol; the whole compile moves to scenomise. Nothing
widens per product; product adapters still resolve their own truth into the
table.

**Ruling S13.** *When does P2 happen?* Options: here, after P1; another lane
now; park it. Mark: **"Here, after P1 (Recommended)"**. Follows: this session
takes P2 once P1 is reviewed.

**Ruling S15 (round 4, from F12).** *Where do authored arrangement ids
resolve?* Options: a built-in catalog beside the registry; register the
built-ins as solvers; ids are sceno's variant names. Mark: **"Built-in catalog
beside registry (Recommended)"**. Follows: scenomise gains a catalog of the
eleven named families, each with an id, a `SolverCapability` (determinism,
requires, tags) and a constructor from the authored arrangement into its typed
sceno variant; an id not in it falls through to the registry as
`Arrangement::Custom`. One lookup for hosts, and the typed enum stays
exhaustive. *Reading, not ruled*: ids are the families' plain names
(`grid`, `spiral`, `geographic` and the rest), and graphshell's
`grid.default` and `scatter.default` resolve as aliases so saved definitions
still load.

**Ruling S16.** *Where does a parameter come from when the definition does not
give it?* Options: measured from the items; documented family defaults;
required in the definition. Mark: **"Measured from the items
(Recommended)"**. Follows: unset parameters derive from the score (a grid
cell fits the largest item, columns follow the item count, spacing is the
authored spacing); options override; no layout constants in hosts or families.

**Ruling S17 (round 5, from F13).** *Who builds the rest of P1, now that the
compiler lives in scenomise under the projection lane's pass?* Options:
coordinate, then build here; hand the rest to that lane; wait for its pass to
end. Mark: **"Coordinate, then build here (Recommended)"**. Follows: this
session tells the projection lane what P1 changes in `scenomise::projection`
and asks it to hold compiler edits until P1 lands, or to name what its
qualification needs; P1 is built in a worktree from current main, and nothing
merges before Mark's review.

**Ruling S18 (round 6, from F13's correction).** *Proceed with P1, now that
S17 reached the wrong lane?* Options: proceed and leave a note; find the owner
first; hand P1 to that owner. Mark (free text): **"Ah, i know the agent. Pass
me a message and I'll relay it."** Follows: a message stating P1's changes
(S12, S15, S16, the moved paths kept re-exported, the id aliases, and the pitch
change from 200 by 100 to 180 by 84) and asking the recipe pass to hold
compiler edits, name what its Woodshed pass needs, and say whether the pitch
change suits its captures went to Mark for relay. *Reading, not ruled*: P1's
code waits for that answer; read-only assessment continues.

**Ruling S19 (round 7).** *Restart P1 now?* The message S18 sent for relay
reached the graph-semantics lane, not the recipe pass, whose author is still
unidentified here; nothing had touched the compiler since `c79bb8c2`, and the
graph-semantics lane was holding the files. Options: build now and leave a
note; wait for the recipe author. Mark: **"Build now, leave a note
(Recommended)"**. Follows: P1 is built in the `stack-seams-p1` worktree; a dated
cross-reference in the projection grammar adoption plan states the moved
paths, the aliases, the signature change and the pitch change for the recipe
pass to find at repin; nothing merges before Mark's review.

**Ruling S20.** *Where does an item's size come from?* The compiler writes
every card in at 164 by 68 (`scenomise/src/projection.rs` 697), which would
make S16's measurement nominal. Options: the host supplies it; an authored
option; a documented default. Mark: **"Host supplies it (Recommended)"**.
Follows: compilation takes the representation's measured size from the host
beside the dataset (graphshell passes its card size, Woodshed its own);
nothing is written into scenomise. This follows the 2026-07-22 charter, which
puts presentation (cards, glyphs) with the host.

**Ruling S21 (round 8, P1's design).** *How do the eleven families get their
per-item inputs?* Recipes encode x, y, color and label; families read an
explicit cell (Grid), a coordinate (Geographic, Hulls), a 2D embedding
(Embedded), a numeric axis (Timeline, Radial's rings), a categorical axis
(Kanban), a layer (Stack), or only order (Spiral, Penrose, LSystem)
(`scenomise/src/families/`, `sceno::ScoreItem`). Options: the catalog maps x
and y; grow the encoding; only where x and y fit. Mark: **"Catalog maps x and
y (Recommended)"**. Follows: each catalog entry declares how it reads the
encoding: Grid ranks x and y into cells; Geographic and Hulls take (x, y) as a
coordinate; Embedded as an embedding; Timeline and Radial take numeric x as
their axis; Kanban takes text x as its column; Stack takes integer x as its
layer; Spiral, Penrose and LSystem order by x. A missing or wrong-typed channel
is a typed issue naming it. All eleven are authorable with today's grammar.

**Ruling S22.** *What shape does the compile API take?* S20 needs the host's
item sizes and S15's fall-through needs a `SolverRegistry`. Options: a compiler
value; extra parameters. Mark: **"A compiler value (Recommended)"**. Follows:
the host builds one `ProjectionCompiler` holding its item sizes and its
registry, and calls `compile`, `refresh`, `compile_snapshot` and the
relationship compile on it. *Reading, not ruled*: the relationship-recipe
compile, which calls `compile_snapshot`, becomes a method too and keeps its
grid-only rule; the alias constants stay exported for saved recipes.

Round 9, 2026-10-05, on P2's design. Evidence: the host is single-root (one
rootstock `Host`, one `native_window`, `run(options, init, hooks)`); texture
producers reach the device through `ProducerContext` and are rebuilt on every
resume, because the producer registry resets when the device changes
(`cambium-rootstock/src/producer/registry.rs` 147) and each resume boots a new
core; the [one state, N windows design](../design/2026-07-05_one_state_n_windows_design.md)
rules "one dom as a forest", and its framework half (`GenetMultiRunner`, now
`crates/cambium/cambium/src/multi.rs`) and the forest dom have landed, unused by
this host.

**Ruling S23.** *Sequence P2?* Options: the shared core first, then windows;
both together. Mark: **"Both together"**. Follows: one change set delivers the
shared, resume-surviving core and multi-window.

**Ruling S24.** *What does a second window show?* Options: one state with N
lenses; independent roots. Mark: **"One state, N lenses (Recommended)"**.
Follows: the host runs `GenetMultiRunner` over one state; each window is a
projection with its own lens, layout session, viewport, DPI and focus, and a
change made in one window reaches the others in the same pass, per the
2026-07-05 design.

**Ruling S25.** *What API do multi-window applications use?* Options: an
additive multi-window entry; one API with a single window as its simplest
case. Mark: **"Additive multi-window entry (Recommended)"**. Follows: a new
entry with a lens per window and a command to open one sits beside `run`,
which stays exactly as it is, so Woodshed, Hocket and Redshank are untouched
and checkpoint C1 is not reached.

**Ruling S26 (C2).** *How do windows pace their frames?* Options: each window
paces itself; one frame clock. Mark: **"Each window paces itself
(Recommended)"**. Follows: each window redraws and presents on its own request,
at its own monitor's rate, the shared core serving them in turn; an idle window
costs nothing. Checkpoint C2 is resolved.

Round 10, 2026-10-05, on how P2 lands. Evidence, from reading the hosts
after S23: the shared core is small (boot one `RenderCore`, make each surface
from it, keep it across resume), but one state with N lenses is not.
Rootstock's `HostState` holds 60 fields, most of them one window's, beside
one `GenetAppRunner`; the owned layout lays out a whole document, and genet's
subtree adapter (`ScopedDom`, `genet-scripted/livery.rs` 1611) is private; no
host drives `GenetMultiRunner`; the winit host makes one window and boots a new
core on each resume. Roughly 1,500 to 3,000 changed lines over several sessions.

**Ruling S27.** *P2 sized: S23's "Both together" was ruled before these numbers.
How should it land?* Options, put in plain text: keep S23, one branch built in
stages with a check-in at each stage and one merge; land the shared core now,
with multi-window as its own later change; land the core and park
multi-window, then do the S14 pass. Mark: **"1. feel free to orchestrate using
subagents."**, then **"oh, i meant #1 of the three options."** Follows: S23
stands; P2 is one branch built in stages, each stage ending with its receipts,
and it merges once; subagents may carry stages.

Round 11, 2026-10-05, on P2's shape. Evidence: `GenetMultiRunner` supports
both topologies (`push_projection`, `push_forest_projection`); the owned layout
is generic over `LayoutDom` (`owned_layout.rs` 86 onward), so a forest needs a
window-subtree adapter, one mutation drain routed by window root, and
accessibility scoped per window with ids salted by window
(`cambium-winit-a11y` reads the whole document at four sites), an estimated
500 to 700 lines more than separate documents; the frame and input pipeline
(`host.rs`, `frame.rs`, `input/`, about 3,100 lines) reads one runner and one
layout; Woodshed, Hocket and Redshank use only `run`, `AppCtx`, `HostHooks` and
`Harness`, never `HostState`'s fields.

**Ruling S28.** *Which document topology do P2's windows use?* Options: the
forest dom (one `ScriptedDom`, one window-root per window; P2 an estimated
1,700 to 2,700 lines in four stages); N doms (each window its own document; an
estimated 1,200 to 2,000 lines in three stages, a cross-window move rebuilding
its content). Mark: **"Forest dom (Recommended)"**. Follows: one document with
a window-root per window, laid out per window through a subtree adapter, so a
`move_before` between windows keeps the node, its scroll and its focus.

**Ruling S29.** *Where do custom-paint leaves and texture producers live once
there are several windows?* Options: one shared registry, each window painting
the leaves its own subtree holds; one registry per window. Mark: **"Shared,
painted per window (Recommended)"**. Follows: the application keeps one
`LeafRegistry` and one `ProducerRegistry`; each window renders the leaves its
layout has boxes for, so a leaf moved to another window keeps its painter.

**Ruling S30.** *Does the single-window `run` move onto the shared per-window
pipeline?* Options: one pipeline, `run`'s signatures unchanged; a separate
multi host beside an untouched `Host`, duplicating about 1,500 lines. Mark:
**"One pipeline (Recommended)"**. Follows: `run` and the multi-window entry
drive the same per-window frame and input code; `run`, `AppCtx`, `HostHooks`,
`Init` and `Harness` keep their signatures, so S25 and C1 hold.

**Ruling S31.** *S27 checks in at each stage boundary. What does a check-in ask
of you?* Options: report, then continue; wait for a go. Mark: **"nah, just
proceed."** Follows: amends S27; the stages run back to back, each ending with
its receipts recorded in §5, and no stage waits at a boundary. *Reading, not
ruled*: a fork with more than one defensible answer still stops for Mark, as
the lanes rule has it, and C1 still stops.

Round 12, 2026-10-05, after P2 landed. Evidence: F19 (the Knot lane's five
recipes at seven cards, 0 by 0 and NaN among them, byte-identical with no
complaint) and F18 and F20 (what P2 leaves unmeasured off Windows).

**Ruling S32 (F19).** *How should the compiler refuse degenerate card sizes?*
Options: a typed compile issue in the list unknown ids and options already
use; `ProjectionCompiler::new` returning a `Result`, a signature change
graphshell and Knot would follow; a debug assertion only. Mark: **"Typed
compile issue (Recommended)"**. Follows: `compile`, `refresh` and the
snapshot compiles report an `items.card` issue when either side of the card
is not a finite positive number; no signature changes, so the Knot lane's
adaptation stands.

**Ruling S33.** *P2 has landed. What next?* Options: the S14 status-versus-code
pass; measuring F18 and F20 on Mark's Linux and macOS machines; stop. Mark:
**"S14 status-vs-code pass (Recommended)"**. Follows: after S32, the pass S14
ruled runs, checking active plans' status lines and done-claims against the
tree. F18 and F20 stay open.

**Ruling S14.** *Where does the next contradiction pass look?* Options: plan
status against code; rulings across plans; sibling repos too; no pass. Mark:
**"Plan status vs code (Recommended)"**. Follows: active plans' status lines
and done-claims are checked against the tree, starting with the documents the
doc audit flags. *Reading, not ruled*: it runs after P1, one lane at a time.

Round 13, 2026-10-06, during the S14 pass. Evidence: phase A's records
(batches 35 to 37) and how the judgment audit treats a record for a path the
early-September snapshot already covers.

**Ruling S34.** *Phase B re-judges 145 plans that already have
early-September records; the judgment audit rejects a new batch record for any
path the snapshot covers, and DOC_POLICY keeps the snapshot unchanged. Where
should the re-judgments live?* Options: parser-shaped records in a new
directory beside the snapshot, outside the coverage count, with no script
change; the judgment audit changed so a dated batch record replaces the
snapshot's record for that path; no per-document records, only a remediation
receipt. Mark: **"Newer record supersedes"**. Follows:
`scripts/mere_doc_judgment_audit.py` takes a batch record for a path the
snapshot covers as superseding the snapshot's record, and coverage and totals
count the batch record; the snapshot file stays unchanged; DOC_POLICY's
addendum says so. Phase B's records are ordinary batch records.
*Reading, not ruled* (2026-10-06): the same holds between batch records,
a later batch's record superseding an earlier batch's for the same path,
since three of phase B's plans (the event model, fact visualization and
insigne proofs plans) were last judged in batches 15 and 22, not the
snapshot; one batch judging a path twice is still an error.

**Ruling S35.** *Three active receipts share the basename `RECEIPT.md`. Rename
them?* Options: rename in place to dated names, updating their citations; the
same with S10's flattened out of its single-file directory; keep the names and
exempt them in the audit. Mark: **"Rename, flatten S10"**. Follows: the
custody-backend receipt becomes
`custody-backend/2026-09-08_custody_backend_receipt.md`, the R3 receipt
`2026-09-20_resource_resolution_probes/2026-09-20_resource_resolution_receipt.md`,
and S10's `mere_docs/testing/receipts/2026-09-09_s10_custody_transact_receipt.md`;
their citations, the custody-backend artifact list's key and batch 37's
headers follow.

**Ruling S36.** *Phase A found three plans complete, and phase B will find
more. What should phase C do with plans found complete?* Options: extract
their open follow-ons to a named plan, then archive; correct the status only
and keep them active; list them for case-by-case rulings. Mark: **"Extract and
archive (Recommended)"**. Follows: in phase C each plan the pass finds
complete has its open follow-ons moved to a named existing or new plan, then
moves to `archive_docs/` with its DOC_README entry, links repaired and both
audits green. *Reading, not ruled*: a plan whose completeness is itself in
question (a record's fork) still goes to Mark first.

**Ruling S37.** *The lexical capability plan says "not started", but
`crates/intel/reference-data` (`9310518b`) carries much of its L2. Is
`reference-data` the plan's L2?* Options: yes, re-scope L2 onto it; two
surfaces, each justified; ask its lane first. Mark: **"Ask its lane first"**.
Follows: the session that built `reference-data` is asked what it intended,
and its answer comes back to Mark as this question's evidence; the lexical
plan's record (batch 35) stands meanwhile.

Round 14, 2026-10-06, during phase B. Evidence: batches 39 to 46 (96 plans)
and the drafts' "Forks for Mark" lists, about 80 so far.

**Ruling S38.** *Phase B has turned up about 80 design forks, most belonging
to their plan's own lane rather than to this status pass. How should they be
handled?* Options: phase C writes each into its plan as a dated "Open, raised
by the S14 pass" note and only forks that block a status correction or an
archive move come to Mark; every fork to Mark in rounds; one forks ledger.
Mark: **"Route to lanes (Recommended)"**. Follows: phase C records each fork
in its own plan with its options; Mark is asked only where a status line or an
archive move cannot be written without the answer.

**Ruling S39.** *About 30 plans describe meerkat code as landed or live; how
does phase C word their status?* Options: landed, then retired with meerkat,
with survivors and open items named; count only surviving library code as
landed; reopen meerkat-only work against Turnstone. Mark: **"Landed, then
retired (Recommended)"**. Follows: such a status reads "landed in meerkat on
<date>, retired with it 2026-07-18 (`c5f01064`); surviving library parts:
...; open: ...", and a plan with nothing else open is extracted and archived
under S36.

**Ruling S40.** *Merge `0a8198ba` (2026-09-06) dropped inker's `LICENSE-MIT`
and `LICENSE-APACHE`, which LICENSES.md says are not to be deleted. Restore
them or accept the deletion?* Options: restore from the merge's first parent;
accept the deletion and amend LICENSES.md. Mark: **"Eh, they’re supposed to be
mpl-2.0"**. Follows: the files stay deleted; inker is MPL-2.0 like the rest of
the workspace, and LICENSES.md's inker and tinct rows and its "not to be
deleted" sentence are amended in phase C. *Reading, not ruled*: the versions
already on crates.io keep the grant they were published under.

**Ruling S41.** *S37 said to ask `reference-data`'s lane, but no reachable
session built it (no Claude transcript has its code; the Knot composition
work beside it sits next to a `codex/knot-composition` branch). Who answers?*
Options: Mark asks that Codex session; the active Knot lane answers; rule it
now. Mark: **"Rule it now"**. Follows: S37's question goes back to Mark
directly, in round 15.

Round 15, 2026-10-06. S37's question put back under S41, and the three
forks phase C cannot write a status line without (S38).

**Ruling S42 (S37 put back).** *Is `reference-data` (`9310518b`: opt-in,
licence-carrying, BLAKE3-checked packs of language-tagged senses and
relations, which Knot pins) the lexical capability plan's L2?* Options: yes,
L2 re-scopes onto it; two surfaces, each justified; retire one. Mark: **"Yes,
it is L2 (Recommended)"**. Follows: phase C records L2 as partly built in
`reference-data` and re-scopes L2 onto that crate; L1, L3, L5 and
Turnstone's L4 stay open; mere has one lexical surface.

**Ruling S43.** *The lattice sync pass's status is dated 2026-09-20; L1 and
L2's passes, the consumer round, CI and the two-row patch baseline have
landed since, and L5 (three knot-editor revisions), the L3/L6/L7 consumers,
P2's cleanup and P4 remain. Continue or close?* Options: continue with the
status rewritten; close and extract. Mark: **"Continue, status rewritten
(Recommended)"**. Follows: phase C rewrites the status to record what landed
and name the four open items; the plan stays active as their owner.

**Ruling S44.** *Three first-party manifests pin `sha2` 0.10 against the
crypto plan's ruled 0.11 row (djinn, the distillery session fixture, Pelt
desktop on genet's 0.10 line). What does the status record?* Options: repin
two and except Pelt; repin all three; accept all three as exceptions. Mark:
**"Repin all three"**. Follows: the crypto generation plan reopens for the
three repins, Pelt's possibly waiting on genet's own move to 0.11; the repins
are a follow-on task, not part of this documentation pass.

**Ruling S45.** *61 owned sources lack the MPL Exhibit A header the license
sweep's P1 audit requires, most carrying only a short SPDX line. What should
happen?* Options: re-run the header tool and add a gate; record the drift
only; accept the SPDX header. Mark: **"Re-run tool, add a gate
(Recommended)"**. Follows: `scripts/relicense_headers.py` is re-run over the
files on a clean tree in one commit, and a check makes new owned files carry
the header; both are a follow-on task outside this pass, and the license
sweep's status records the drift until then.

Round 16, 2026-10-06, after phase B. Evidence: the phase C assessment in
§5 (166 documents with recommended actions, about 67 archive candidates, 27
documents touched by other lanes since 2026-09-29).

**Ruling S46.** *About a dozen of the 166 documents are lanes' live plans
that moved heavily this week. How does phase C treat them?* Options: hand
them to their lanes with their records and correct the rest; edit all of
them additively; ask each lane first. Mark: **"Hand to their lanes
(Recommended)"**. Follows: phase C leaves the live plans alone and each lane
is pointed at its record; the other documents are corrected. *Reading, not
ruled*: "live" means ten or more commits from other lanes since 2026-09-29,
plus the djinn test harness plan (founded 2026-10-05, its lane active):
physics catalog, dynamics grammar, burn 0.22 migration, graphshell one tree,
vault lock, device pairing, chatelaine, projection grammar adoption, the
controls and physics receipt, conatus engine, djinn test harness.

**Ruling S47.** *How are the corrections carried out?* Options: opus write
lanes in their own worktrees, each diff reviewed with both audits green before
it merges; sequentially by this session. Mark: **"Write lanes, reviewed
(Recommended)"**. Follows: write lanes correct their assigned documents in
worktrees; this session reviews each diff, runs both audits, and merges.
*Reading, not ruled*: lanes edit only their assigned documents; DOC_README,
archive moves and the tails backlog are cross-cutting, so lanes propose them
and this session applies them centrally, avoiding merge collisions.

**Ruling S48.** *Where do archived plans' ownerless tails go?* Options: their
live owner when one plainly exists, else a new dated section of the archived
plan tails backlog; a new follow-ons plan per area root; ask per plan. Mark:
**"Archived plan tails (Recommended)"**. Follows: phase C moves each tail to
its plain live owner or into a dated section of
`2026-07-03_archived_plan_tails_plan.md`, citing the archived plan, before
the archive move.

Round 17, 2026-10-06, after phase C's fourteen lanes merged. Evidence: the
lanes' archive-candidate lists; 46 plans plainly complete, about 53 posing
completeness as a fork.

**Ruling S49.** *About 53 plans pose completeness as a fork, most asking
whether a meerkat-era or dormant plan's open items are re-homed onto a live
host or archived. How are they handled?* Options: leave them active with
their forks as Open notes, lanes deciding later; rule them now in
multi-select rounds; archive only the unbuilt ones without a consumer. Mark:
**"Rule them now"**. Follows: the plans come to Mark in multi-select rounds;
each ticked plan is archived with its tails extracted (S36, S48), and each
unticked plan stays active with its corrected status. The 46 plainly complete
plans are archived under S36 without a round.

Round 18, 2026-10-06. Evidence: phase C's per-plan open items for the first sixteen of the plans S49 brings to rulings; each question was multi-select, a ticked plan archived with its open items extracted, an unticked one staying active.

**Ruling S50.** *Meerkat-era engine and flip plans: which are archived?* Options: the engine picker; the scrying tile; the genet scrying flipcarrier; the render ladder (multi-select). Mark: **"render ladder needs rethinking.,Scrying tile,Render ladder"**. Follows: the scrying tile and render ladder plans are archived, their open items to the archived plan tails backlog, the render ladder's carrying Mark's note that it needs rethinking; the engine picker and the flipcarrier stay active.

**Ruling S51.** *Meerkat-era feature plans: which are archived?* Options: the comms shell; tear-out gestures; portable tiles; overlay roots and UA widgets (multi-select). Mark: **"Comms shell,Tear-out gestures"**. Follows: the comms shell and tear-out gestures plans are archived, open items to the backlog; portable tiles and overlay roots stay active.

**Ruling S52.** *Canvas and graph-view plans: which are archived?* Options: the swatch primitive; orrery graph intelligence; the orrery custom layout element; node navigation lineage (multi-select). Mark: **"Orrery graph intelligence,Orrery custom layout element"**. Follows: the orrery graph intelligence and orrery custom layout element plans are archived, open items to the backlog; the swatch primitive and node navigation lineage stay active.

**Ruling S53.** *Graph data and RDF plans: which are archived?* Options: the graph query layer; petgraph RDF; the graph write-path migration; graph delta capture and stats (multi-select). Mark: **"Graph query layer,Petgraph RDF,Graph write-path migration,Graph delta capture / stats"**. Follows: all four are archived: the graph query layer's #3 and Graph-results error to graph semantics P4 and its other residue to the backlog; petgraph RDF's open items to the graph semantics plan; the write-path plan's apply.rs split and the capture plan's regression log and per-table stats to the backlog.

Round 19, 2026-10-06. Evidence: phase C's per-plan open items for the next sixteen of S49's plans; multi-select, a ticked plan archived with its open items extracted, an unticked one staying active.

**Ruling S54.** *Dormant or unbuilt plans: which are archived?* Options: the engine profile boundary; the session service runner; the bounty verification economy; the polyglot block resolver (multi-select). Mark: **"Engine profile boundary, Session service runner, Bounty verification economy"**. Follows: those three are archived, their v0b scopes and the bounty design to the archived plan tails backlog; the polyglot block resolver stays active.

**Ruling S55.** *Unbuilt design plans: which are archived?* Options: the runtime mod authoring loop; MCP native graph; persona transport unlinkability; capture provenance consent (multi-select). Mark: **"Capture provenance consent"**. Follows: capture provenance consent is archived, its C4 consent gate going to the live recorders' owners (Turnstone's trail memory and page capture plan, Graphshell H5) and its other tails to the backlog; the other three stay active.

**Ruling S56.** *Identity and dramatis plans: which are archived?* Options: the SSH CA projection; insigne proofs; the identity vault SSH agent; castellan OTP (multi-select). Mark: **"Insigne proofs, Identity vault SSH agent, Castellan OTP"**. Follows: those three are archived: insigne's remaining repins and castellan OTP's follow-ons to the dramatis repo plan or the backlog, the vault's V4 to the chatelaine plan and V5 to the backlog; the SSH CA projection stays active.

**Ruling S57.** *More identity and protocol plans: which are archived?* Options: the dramatis tier; the persona wallet carry layer; the protocol architecture; participant gate packs (multi-select). Mark: **"Dramatis tier, Persona wallet carry layer, Participant gate packs"**. Follows: those three are archived: the dramatis facade and the wallet's carry gaps to the dramatis repo plan (lock and unlock to the vault lock plan), the gate packs' B5 tessera receipt and meadowcap layer to the backlog; the protocol architecture stays active.

Round 20, 2026-10-06. Evidence: phase C's per-plan open items for the next sixteen of S49's plans; multi-select, a ticked plan archived with its open items extracted, an unticked one staying active.

**Ruling S58.** *Distillery and ML plans: which are archived?* Options: distillery v0; the inference provider; the browser model ceiling probe; mesh host lanes (multi-select). Mark: **"Distillery v0, Inference provider, Browser model ceiling probe, Mesh host lanes, These four need to be consolidated now that 0.22 is out"**. Follows: all four are archived, and their open items are consolidated into one new plan rather than the backlog. *Reading, not ruled*: "0.22 is out" means stable Burn 0.22 has been released, which the burn plan's stable closure and the mesh host lanes' last gate wait on; the consolidated plan's name and scope come back to Mark.

**Ruling S59.** *Search, memory and receipt plans: which are archived?* Options: eidetic browsing derivation; redb over OPFS feasibility; the intel vector index burn lift (mere twin); receipt artifacts replication (multi-select). Mark: **"Eidetic browsing derivation, redb over OPFS feasibility, Intel vector index burn lift (mere twin), Receipt artifacts replication, These also need a good critical pass and to be turned into a plan if needed, but eidetic needs development now that identity has a clear path"**. Follows: all four are archived; their open items get a critical pass and become a plan if warranted, and Eidetic's development is to be taken up now that identity's path is clear. *Reading, not ruled*: the critical pass is a new objective with its own assessment, after phase C.

**Ruling S60.** *Host and surface plans: which are archived?* Options: the workbench component; the knot shared surface; host UI zoom; component catalog growth (multi-select). Mark: **"Workbench component, Knot shared surface, Host UI zoom, Component catalog growth"**. Follows: all four are archived: the workbench's S3 to Turnstone's pane registry plan (A4), the shared surface's F0 to knot-editor's application workspace plan and its pointer-capture gap to Turnstone, the zoom plan's Z5 figure to Isometry's migration plan, and the remaining tails (catalog promotion rules and decisions among them) to the backlog.

**Ruling S61.** *Projection and publishing plans: which are archived?* Options: projection receipts; native smolweb rendering; the graphshell remote projection host; the knot publishing protocol (multi-select). Mark: **"Projection receipts, Native smolweb rendering, Graphshell remote projection host, Knot publishing protocol, Explain the difference for 2"**. Follows: all four are archived: projection receipts' FT9/FT10 gates to the projection grammar adoption plan, the remote projection host's G6/G7 to the reference host plan's H-series, Knot's copy of the publishing protocol canonical; native smolweb rendering's question of which render design stands (its "2") is explained to Mark and put back before its host-integration tail is placed.

Round 21, 2026-10-06. Evidence: the code of the two smolweb render designs (`crates/cambium/cambium/src/nematic.rs` and `views.rs`; `crates/system/document-lanes/src/smolweb.rs`), and phase C's open items for the last six of S49's plans.

**Ruling S62.** *Native smolweb's two render designs: per-format Cambium views over Errand's ASTs (no host uses them), or the EngineDocument lane (Nematic lowers every format into Inker blocks that document-canvas lays out; Pelt, Turnstone and Signalman read it). Which stands?* Options: the lane as default with bespoke views only where the line model is not box-shaped; the EngineDocument lane only; the per-format views. Mark: **"Wait, the fidelity work can enrich the asts that feed engine documents? Why wouldn’t we do that?"**. Follows: not yet settled. Answered: yes, enriching the ASTs that feed EngineDocument is the fidelity plan's own central finding ("the losses are at the parse ASTs") and applies under every option; what remains open is a format whose structure blocks cannot carry even with richer ASTs (gopher's typed column), which needs a new block kind or a bespoke view. Put back in round 22.

**Ruling S63.** *S58's four ML plans consolidate now that Burn 0.22 is out. Where do their open items go?* Options: a new plan written now; parked in the tails backlog for the ML lane. Mark: **"New plan, written now (Recommended)"**. Follows: this pass writes one new plan carrying the four plans' open items, stable Burn 0.22 adoption first, each item citing its source plan, with phases and done-conditions.

**Ruling S64.** *Architecture and record plans: which are archived?* Options: spatial compute; the tactile tier; derived faces; event model convergence (multi-select). Mark: **"Spatial compute, Tactile tier, Derived faces, Event model convergence"**. Follows: all four are archived: spatial compute's Nexus watch to the conatus engine plan and its other tails to the backlog; the tactile tier's T4 and T5 to the backlog (T5's overlap with dynamics grammar G4 and the physics catalog noted); derived faces' editing, branding gap and manifest question to the backlog; the event model's subclasses and native currentTarget to the backlog, genet's copy then standing as the live one.

**Ruling S65.** *The last two: which are archived?* Options: reticulum transport; doc policy consolidation (multi-select). Mark: **"Reticulum transport, Doc policy consolidation"**. Follows: both are archived: reticulum's forged legacy-binding test, README docs and Phase 3 decision to the backlog (retinue's plan covers the backend); the doc policy plan's genet docs/ merge to the backlog, with DOC_POLICY.md's "Track the work in" pointer repointed to DOC_POLICY itself.

Round 22, 2026-10-06. Evidence: S62's reframing: enriching the ASTs that feed EngineDocument applies under every option; open only for structure blocks cannot carry (gopher's typed column, feed entries and enclosures).

**Ruling S66.** *For smolweb structure that EngineDocument's blocks cannot carry even from a richer AST, which?* Options: extend EngineDocument with the block kind it lacks, one path, the per-format views retiring; keep bespoke per-format views for those exceptions. Mark: **"Extend EngineDocument (Recommended)"**. Follows: the EngineDocument lane is the single smolweb render path: the fidelity plan enriches the parse ASTs and adds whatever block kind EngineDocument lacks (a typed-column block first), so every host gets it through the lane; the per-format views in `cambium::nematic` retire (a follow-on code task); the native smolweb rendering plan's two-family model is historical, and the plan archives under S61 with its host-integration tail carried by the fidelity plan.

Round 23, 2026-10-06. Evidence: the phase C remediation receipt (`support/doc-audit/d2/remediation_2026-10-06.md`) and its readings not ruled; knot-editor carries near-identical copies of the Knot port, authoring consumer and knot-in-graphshell plans; about ten archived tails name plans in Turnstone, knot-editor and Isometry, whose sessions are offline.

**Ruling S67.** *Who owns the tails of the three archived Knot plans (the knot port, the authoring consumer, Knot in graphshell): the 'one immutable revision' pin rule, the optional headed selected-clip receipt, Knot search S0/S1?* Options: knot-editor's copies, as S61 ruled for the publishing protocol; mere's backlog until a Knot plan names them. Mark: **"knot-editor's copies (Recommended)"**. Follows: knot-editor's copies own them; the tails section's `owner: knot-editor's copy` entries stand, and the receipt's reading is now ruled.

**Ruling S68.** *Petgraph RDF's Phase 4 is gated and was not in that plan's Open list; S53 sends its open items to the graph semantics plan. Does Phase 4 go there too?* Options: the backlog, as recorded; a Received note in the graph semantics plan. Mark: **"Backlog (Recommended)"**. Follows: Phase 4 stays in the archived plan tails backlog; the receipt's reading is now ruled.

**Ruling S69.** *About ten archived tails name plans in other repositories (Turnstone: the page capture C4 gate, the pane registry's A4, the pointer-capture gap; knot-editor: six items; Isometry: the Z5 figure). Their sessions are offline. What happens to them?* Options: record only, in mere's tails section; a dated note in each owning plan, widening scope to the three repositories; message each session when it is next online. Mark: **"Note in each plan"**. Follows: each owning plan in Turnstone, knot-editor and Isometry gets a short dated note naming its tail and citing mere's tails section and the archived plan; each repository's DOC_POLICY and git posture are read first, and the note is the only edit there.

**Ruling S70.** *What is next now that the S14 pass is done?* Options: the S59 critical pass with Eidetic's development; S66's code follow-on (a typed-column block in EngineDocument, richer smolweb ASTs, the per-format `cambium::nematic` views retiring); the render ladder rethink; the F18/F20 measurements. Mark: **"S66 code follow-on"**. Follows: S66's code follow-on is the next objective, assessed first (the smolweb fidelity plan, EngineDocument, the views' consumers) before any code.

Round 24, 2026-10-06. Evidence: S70's assessment. Nothing in any repository calls the per-format views (`gemtext_view`, `gopher_view`, `nex_view`, `feed_view`) and nothing enables cambium's `nematic` feature. Every host renders smolweb through document-canvas (`SmolwebDocument` holds Nematic's `EngineDocument`), which already has monospace text and a table grid. Nematic lowers gopher links to plain paragraphs and info lines to one preformatted block, losing each row's type. `InlineSpan::Submit` exists (the Spartan prompt uses it). Gemtext keeps line boundaries as `SoftBreak`, which document-canvas renders as a space (`text.rs:218`), so the hard-break setting is render-side. `Block` is not `#[non_exhaustive]`; about ten renderers in Mere and knot-editor's desktop preview match it exhaustively. gopher-protocol (smolweb repository, crates.io, pinned `=0.1.1`) has no raw item type, keeps the 8/T inversion and has no CSO kind; Errand and Nematic are published, so they cannot take a git dependency. Errand's `FeedEntry` has guid, link, one date, one summary and enclosures, no content; `Block::FeedEntry` has title, date, summary and two URLs.

**Ruling S71.** *What shape does EngineDocument's typed-column block take?* Options: a typed menu block (new `Block::Menu`: rows with an item kind, the raw type character, label spans and an optional target, laid out as a fixed-width grid, a list of typed items to accessibility); `Block::Table` plus a fixed-width flag on `BlockPresentation`; a fixed-width wrapper only. Mark: **"Typed menu block (Recommended)"**. Follows: `Block::Menu` is added: each row carries an item kind, the raw type character, label spans and an optional target; info rows have no target and a search row carries `InlineSpan::Submit`; document-canvas lays it out as a fixed-width grid (type column, label column) and the accessibility projection exposes a list of typed items. Gopher lowers to it first.

**Ruling S72.** *Adding a Block kind breaks every exhaustive match outside inker (about ten renderers in Mere, knot-editor's desktop preview). Does Block become `#[non_exhaustive]` in the same change?* Options: keep it exhaustive, the compiler finding every renderer, knot-editor briefly red first; mark it `#[non_exhaustive]`, every crate outside inker gaining a wildcard arm. Mark: **"Mark non_exhaustive"**. Follows: `Block` becomes `#[non_exhaustive]` in the same change; every crate outside inker, Mere's own renderers included, gains a wildcard arm, and knot-editor and Turnstone take that change once and then stop breaking on new block kinds. *Reading, not ruled*: a wildcard arm surfaces the kind it cannot draw as a `DocumentDiagnostic::UnsupportedConstruct` rather than dropping it silently, so the compiler check S72 gives up is replaced by a visible one.

**Ruling S73.** *A faithful type column needs gopher-protocol to gain a raw item type, the 8/T fix and a CSO kind: a breaking 0.2.0 release, and publishing is irreversible. How is it sequenced?* Options: coarse now, release later (the block carries a raw-type field from day one, the crate changes prepared in the smolweb repository and published only on Mark's sign-off); release first; coarse kinds only. Mark: **"Coarse now, release later (Recommended)"**. Follows: `Block::Menu` carries the raw type character from the start, filled from today's kinds (`Other(char)` keeps unknown characters; a known kind maps to its canonical character); gopher-protocol's raw type, 8/T fix and CSO kind are prepared in the smolweb repository, and 0.2.0 is published only on Mark's sign-off, after which Mere repins.

**Ruling S74.** *Is the feed part of this objective? Errand's feed tree and `Block::FeedEntry` would gain published and updated dates, a separate summary and content, and enclosures; content is HTML, so how the article reader renders it is an open lane decision.* Options: the fields now and the reader later; the typed column only; the fields and the reader. Mark: **"Fields and reader"**. Follows: the feed is in this objective: errand's feed tree and `Block::FeedEntry` gain published and updated dates, summary and content separately, and enclosures, in the same breaking change as `Block::Menu`; the article reader's lane is decided now, as the next round.

Round 25, 2026-10-06. Evidence: S74 put the article reader in scope. Errand gains `content` (HTML) under S74; Nematic's default `html-fragment` engine (`crates/nematic/nematic/src/html.rs`) already lowers HTML into EngineDocument blocks reader-mode style, passing only a passive subset (no scripts, frames, forms, styling authority or active URL schemes); hosts today follow an entry's article URL to the web; document folds hang off headings only (`inker/src/document/navigation.rs`).

**Ruling S75.** *How does a feed entry's article body render?* Options: its own document (the entry's content lowered through the html-fragment engine, addressed as the feed plus the entry's guid, the article URL its canonical link); inline in the feed document, folded closed; handed off to the web lane. Mark: **"Own document (Recommended)"**. Follows: opening an entry renders its `content` through Nematic's html-fragment engine as its own EngineDocument, addressed as the feed plus the entry's guid, with the article URL as its canonical link; it reads offline on the one path, and an entry without content shows its summary and a link out.

Round 26, 2026-10-06. Evidence: WS4 R0 to R5 merged to main as `b4f14f2c` with every gate green. R6 sits on the smolweb branch `gopher-protocol-0.2.0` (`b0f1dda`): a raw item type, `8` and `T` as telnet, a CSO kind, 39 tests passing and the smolweb workspace clean. knot-editor's desktop preview matches `Block` exhaustively and needs a wildcard arm and a `..` in its `FeedEntry` pattern at its next repin; the fidelity plan's stop rules send changes in knot-editor to Mark.

**Ruling S76.** *Publish gopher-protocol 0.2.0 (irreversible), after which Mere repins and Nematic's gopher engine reads the raw type?* Options: publish 0.2.0; merge into smolweb main without publishing; hold the branch. Mark: **"Publish 0.2.0 (Recommended)"**. Follows: the branch merges into smolweb main, gopher-protocol 0.2.0 is published to crates.io, Mere repins to `=0.2.0`, and Nematic's gopher engine takes each row's marker from `raw_type`.

**Ruling S77.** *knot-editor's desktop preview needs a wildcard arm and a `..` in its `FeedEntry` pattern at its next repin past `b4f14f2c`, and could draw `Block::Menu`. Who does it?* Options: a dated note in knot-editor's application workspace plan, their lane doing it at the repin; this session makes the change in knot-editor; leave it until the repin goes red. Mark: **"Note it in their plan (Recommended)"**. Follows: a dated note in knot-editor's application workspace plan names the change and cites the fidelity plan's WS4; knot-editor's lane makes it at its next repin.

Round 27, 2026-10-06, carrying out S45. Evidence, at `ea74604b` with the tree clean: the license sweep's grep finds 68 owned `.rs` files without Exhibit A, S45's 61 plus 7 added since (speed and runtime files in pictograph, seiche and graphshell). `scripts/relicense_headers.py`, which also covers `.py`, `.js` and `.ps1` under the sweep's invariant 3, flags 83: 13 Python, 1 JavaScript and 1 PowerShell file more, all Mark's, with no third-party copyright line in any. Eight of them sit in `design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes`, whose `artifact-sha256.json` files pin each one's bytes (23 of 23 and 9 of 9 recompute) and whose `.gitattributes` keeps bytes so the digests survive checkout. An index-based check (`git grep`) takes about a second against `--audit`'s forty. CI (`portable.yml`) has failed its last six runs at its second step, `test_cargo_mode.py`, skipping every later step. `.git-blame-ignore-revs` is scoped to rustfmt sweeps and never listed P1's header sweep.

**Ruling S78.** *Which files does the S45 sweep cover?* Options: everything the tool flags (83 less S79's exemption, the gate checking all six extensions as `--audit` does); Rust only (68 less the exemption, the 15 scripts left unheaded). Mark: **"All the tool flags (Recommended)"**. Follows: every owned tracked source in the tool's six extensions takes shape C except S79's directory, and the gate checks all six.

**Ruling S79.** *Eight flagged files are receipt sources in `2026-09-08_stack_pillar_probes` whose bytes the receipt pins by SHA-256. How are they treated?* Options: exempt as frozen evidence (a new LICENSES.md section the tool reads as a second skip list, MPL-2.0 through the root `LICENSE`, the unpinned 2026-09-20 probes taking headers); header them and re-record the hashes; exempt all of `testing/receipts`. Mark: **"Exempt as frozen evidence (Recommended)"**. Follows: LICENSES.md gains a Frozen evidence section naming that directory, the tool skips it as it skips Retained licenses, and its receipts keep recomputing; the 2026-09-20 resource resolution probes, which pin nothing, take headers.

**Ruling S80.** *Where does the gate run?* Options: a CI step placed before the failing test, plus a finding category in the doc audit; CI only; the doc audit only. Mark: **"CI first step + doc audit (Recommended)"**. Follows: `relicense_headers.py --check` (index-based; exit 1 on an owned source without Exhibit A or with the Exhibit B notice) and `--self-test` (planted defects must fail, a clean fixture must pass) run as `portable.yml`'s first checks, and `mere_doc_audit.py` reports both as finding categories. *Reading, not ruled*: the two categories count toward `--fail-on-findings`, and the doc audit's self-test plants one of each.

**Ruling S81.** *Does the sweep's hash go into `.git-blame-ignore-revs`?* Options: leave it out (the file's stated rustfmt scope, P1's precedent); add it, widening the file to header sweeps. Mark: **"Leave it out (Recommended)"**. Follows: the file is unchanged.

Round 28, 2026-10-06, during the S45 sweep. Evidence: before `--apply`, the tree was searched for every flagged file's SHA-256 and git blob id, whole and as 12-character prefixes, with S79's eight files as the positive control (all eight found, at 16 sites). Two sweep files are recorded in graphshell receipts' `source_sha256` maps of 2026-09-06: `ports/graphshell/web/co_op.js` (`co_op_browser_receipt.json`) and `ports/pelt/core/src/surface_policy.rs` (`projection_refresh_surface_reuse_receipt.json`). Both are live sources, and both maps are snapshots that had already gone mostly stale: 7 of 10 and 22 of 30 entries no longer recompute.

**Ruling S82.** *How are the two live sources these receipts record treated?* Options: sweep them, the receipts staying as records of the 2026-09-06 state, "with 9 of 10 and 24 of 30 entries stale"; hold them out as frozen evidence; sweep them and annotate the receipts README. Mark: **"Sweep them (Recommended)"**. Follows: both take the header with the other 73, and Frozen evidence stays for frozen sources only. *Erratum*: the option's figures were miscounted by one each. After the sweep the maps stand at 8 of 10 and 23 of 30 stale (recomputed at `43353861`), one more entry each, which is what the option meant.

## 3. Phases

### 3.1 This plan's lane

- **P1. One arrangement catalog (S1, S11, S12).** Graphshell's generic
  dataset (`ProjectionDataset`, `ProjectionOccurrence`, `ProjectionFieldType`,
  `ProjectionValue`) moves to scenograph as a host-neutral type, documented as
  the in-host input to compilation and never part of the remote protocol. The
  whole compile (`compile`, `refresh`, `compile_snapshot` and their issue and
  result types) moves to scenomise, which holds the registry and already
  depends on sceno: it takes a scenograph definition and a dataset and returns
  a `sceno::Score`, resolving the arrangement id through the registry with
  parameters from the definition. The arrangement resolves through S15's
  built-in catalog, falling through to the registry for custom ids, with unset
  parameters measured from the items (S16).
  Graphshell's `arrangement_for` and `placement_for` retire into it.
  Scenograph's crate doc is corrected. **Owner:** this plan's lane. Asked
  through the projection grammar session, which works beside
  `projection_compile`, Mark answered "Seams lane keeps P1 (Recommended)"; that
  session recorded the answer and S1 in the
  [projection grammar adoption plan](2026-08-15_projection_grammar_adoption_plan.md)
  at `1a2e71db`, and reports no lane of its own in the file (last touched
  `e387df16`, 2026-09-09). *Reading, not ruled*: scenomise is the
  home because scenograph deliberately has no solver dependency.
  Done when: every family the registry holds compiles from a definition (a
  test iterates the registry); graphshell's existing projection tests pass
  through the shared step; an unregistered id fails with a typed error; no
  written-in cell size or column count remains in graphshell; a second caller
  (a test host) compiles the same definition to the same `Score`.
- **P2. One device, many windows (S2).** `cambium-genet-winit-host` boots one
  `RenderCore`, makes every window's surface with `from_shared_core`, exposes
  the core to tenants, keeps it across suspend and resume, and opens more than
  one window.
  Done when: two windows render with one device created (a counter on device
  creation asserts one); a tenant handed the core sees the same device; a
  suspend and resume creates no device; mere's consumers (mere-view,
  cambium-rootstock, cambium-genet-web-host, pelt desktop) build and their
  tests pass. Woodshed's three consumers live in another repository: an API
  change they must follow is a stop (checkpoint C1).
  Staged by S27 to S31 on one branch (`stack-seams-p2`), merged once:
  - **Stage 1, shared core (winit host).** One `Arc<RenderCore>` booted at the
    first resume from `HostOptions::netrender` and kept for the host's life;
    each surface made from it with its transparency; a resume makes a new
    surface and no new core; a count of core boots; the core reachable from
    host state and `AppCtx` for tenants. Done when a forced suspend and resume
    leaves the boot count at one, a tenant's device is the surface's device,
    and the consumers' tests pass.
  - **Stage 2, per-window split (rootstock, no behaviour change).**
    `HostState` divides into what the application shares (resources, leaf and
    producer registries, the core and its fragment ids, commands, wake) and a
    per-window state (window, surface, layout, zoom, pointer, hover and focus,
    accessibility, scroll fade, geometry, captures, profiles); the frame and
    input pipeline runs over one window's state and a crate-internal tree
    trait that `GenetAppRunner` and a multi-runner projection both implement.
    Done when rootstock, the winit host's `Harness` tests, the web host,
    pelt desktop and mere-view pass with no assertion changed, the web host
    checks on wasm32, and a diff of public signatures shows `run`, `AppCtx`,
    `HostHooks`, `Init` and `Harness` unchanged.
  - **Stage 3, forest window sessions (rootstock).** A subtree adapter that
    presents a window-root as its document; each window's layout, hit testing,
    caret, scroll and accessibility read through it; one mutation drain per
    frame routed by window root, a mutation it cannot place dirtying every
    window; accessibility ids salted by window; the document-wide walks scoped
    to the window root; leaves painted per window from the shared registry.
    Done, in a windowless multi-window harness, when two windows over one
    document lay out at different sizes and scales; a mutation in one rebuilds
    that window's layout and not the other's (a mutation in the other moves its
    count: the control); a click in one changes what the other shows in the
    same pass; accessibility ids never collide; and a node moved between window
    roots by `move_before` keeps its `NodeId`, is laid out by its new window,
    and a leaf moved with it keeps its painter.
  - **Stage 4, multi-window entry (winit host).** A `run_windows` entry beside
    `run`: its init returns the state, a lens per window key, the sheet and
    the resources; commands open and close a window; events route by
    `WindowId` to their projection; each window redraws and presents on its
    own request (S26) and an idle one presents nothing; hooks know which window
    they run for. Done when P2's done-conditions above hold with two windows
    (one boot counted, the tenant's device the surfaces' device, a suspend and
    resume creating none), a headed receipt on Windows shows two windows at
    different sizes where a click in one changes the other and closing one
    leaves the other running while an idle window's present count holds still,
    and the workspace checks with `--locked`.
  Merging: main merged into the branch, every gate rerun on the merged tree,
  then main fast-forwards; not pushed.
- **P3. Words (S5, S6).** Done as documents in the commit that records this
  plan: TERMINOLOGY gains **arrangement**, **forme** and **world** and the sceno
  entry is amended. Code identifiers migrate when a file is touched for other
  reasons, never as a churn pass.

### 3.2 For other lanes

**Carried, 2026-10-04.** The dynamics grammar lane took S3 to S6 into its plan
on main: S3 and S4 as inputs with tracks G8 (determinism, all five S4
done-conditions and checkpoint C3, this plan's two probes named as its
instruments) and G9 (permitted actions) at `6c3dca60`; S5 and S6 as inputs,
with G4's binding naming `AdvertisedAction`s and S4's three sum sites, at
`e932d526`. **S9** went to the identity lane, which owns personae and mien,
and is done: mien's type is `PersonaKey` (`f702ca27`, merged on main as
`b52edea7`), with no serde, wire or persisted format carrying it and no user
outside mien. That lane also renamed mien's internal `standing_persona_id`,
which returns the key, to `standing_persona_key`, marking it as its own
reading of the ruling.

The done-conditions handed over for S3 and S4, kept for reference:

- **S3.** The binding's permitted actions are `AdvertisedAction`s; drag and pin
  advertise as `Curation`; an accessibility or permission surface lists them
  with no physics-specific path.
- **S4.** The `seiche-repeat` probe prints one fingerprint across five
  processes (today's five different ones are the control); each `Term`
  declares its determinism and an undeclared one does not compile, like G1's
  declarations; `enhanced-determinism` is on; the probe's fingerprint matches on
  Windows and on macOS or Linux; the cost of `libm` in seiche's laws is
  measured at the probe's sizes and reported.

## 4. Checkpoints

- **C1 (P2).** If multi-window needs an API change the woodshed repository's
  hosts must follow, stop and bring the change to Mark before making it.
- **C2 (P2).** Whether windows share one swapchain cadence (one frame clock for
  all windows) or each window paces itself. Ruled: S26, each window paces itself.
- **C4 (P2).** A change any stage needs in genet or another repository (a
  public `ScopedDom`, a runner API) stops and comes to Mark.
- **C5 (P2).** If a stage-2 or stage-3 change alters what an existing
  single-window application sees (a `Harness` assertion that must change), stop
  and bring the difference back.
- **C3 (S4, dynamics lane).** If `libm` in seiche's laws costs more than the
  run-to-run spread at 5,000 bodies, bring the figure back before keeping it.

## 5. Progress

- **2026-10-04.** Note verified against the code (§1, two of its claims
  stale); rulings S1 to S6; probes kept under `Code/testing/mere/`; P3's
  documents landed with this plan.
- **2026-10-04.** S3 to S6 sent to the dynamics grammar lane, which carried
  them (`6c3dca60`, `e932d526`); S1 sent to the projection grammar lane, which
  works in `projection_compile`. Second pass (F8 to F11), rulings S7 to S10:
  the README fixed (S7), TERMINOLOGY gains pandect under Eidetic and the
  curation record (S8, S10), S9 sent to the identity lane.
- **2026-10-06.** Round 18: S50 to S53, the first sixteen of S49's plans ruled; ten to archive (scrying tile, render ladder, comms shell, tear-out gestures, orrery graph intelligence, orrery custom layout element, graph query layer, petgraph RDF, graph write-path migration, graph delta capture), six to stay active.
- **2026-10-06.** Round 19: S54 to S57, the next sixteen of S49's plans ruled; ten to archive (engine profile boundary, session service runner, bounty verification economy, capture provenance consent, insigne proofs, identity vault SSH agent, castellan OTP, dramatis tier, persona wallet carry layer, participant gate packs), six to stay active.
- **2026-10-06.** Round 20: S58 to S61, all sixteen archived, with three directions: the four ML plans consolidate into one new plan now that Burn 0.22 is out; the four search and memory plans get a critical pass, Eidetic's development taken up; native smolweb's render-design question explained and put back.
- **2026-10-06.** Round 21: S62 to S65. S62's smolweb question answered with Mark's own reframing (enrich the ASTs either way) and put back; S63 a consolidated ML plan written now; six more plans archived (spatial compute, tactile tier, derived faces, event model, reticulum, doc policy). S49's rounds are done: 42 of 53 plans to archive, 11 stay active.
- **2026-10-06.** Round 22: S66, smolweb renders through one path: richer ASTs into an extended EngineDocument, the per-format Cambium views retiring.
- **2026-10-06.** The S14 pass, phase C done; the pass is complete. Fourteen
  write lanes corrected 145 of their 155 documents (the other ten needed no
  edit), merged `3e107e32` to `a3b69537`: 428 dated corrections in 134
  documents and 75 Open notes; the live plans' lanes corrected eleven more
  (S46). DOC_README lane D (`d2184520`) corrected 99 entries and indexed 9
  orphans. Archive lane E (merged `8d7702d2`) moved 88 plans (77 completed,
  7 superseded, 4 retired), rewrote 674 links, and recorded about 200 tails in
  the archived plan tails plan's "2026-10-06 archive pass" section, the S59
  critical-pass subsection among them; the identity and Conatus lanes took
  their `owner:` tails the same day (`73812e3a`, `a0cfa30a`). The receipt is
  `support/doc-audit/d2/remediation_2026-10-06.md`: per-lane counts, the
  readings not ruled (knot-editor's copies as owners, petgraph RDF Phase 4
  to the backlog), six record errata and the follow-ons (S44, S45, S59, S66's
  code retirement, the render ladder rethink). Both audits exit 0; judgment
  coverage 253/253.
- **2026-10-06.** Round 23: S67 (knot-editor's copies own the Knot tails), S68 (petgraph RDF Phase 4 stays in the backlog), S69 (a dated note in each other repository's owning plan) and S70 (S66's code follow-on next).
- **2026-10-06.** S69 done: dated notes in Turnstone (`a616ece`: the C4
  consent gate in the page capture plan; workbench S3 and the pointer-capture
  gap in the pane registry plan), knot-editor (`33cc855`: five plans, the
  Knot tails under S60, S61 and S67) and Isometry (`929647b8`: Z5's figure).
  *Reading, not ruled*: the two tails without a named plan went to the plan
  owning their code path. Turnstone's untracked `.github/workflows/portable.yml`
  (2026-09-20) was left alone. The projection grammar lane took FT9/FT10 as
  its adoption plan's Track F (`28985298`).
- **2026-10-06.** Round 24: S71 to S74, S66's code follow-on shaped: a typed `Block::Menu`, `Block` made `#[non_exhaustive]`, gopher-protocol's raw types prepared and released on sign-off, the feed fields and the article reader in scope.
- **2026-10-06.** Round 25: S75, a feed entry's article renders as its own EngineDocument through the html-fragment engine.
- **2026-10-06.** S70's objective landed: the smolweb fidelity plan's WS4,
  R0 to R5, merged as `b4f14f2c` (per-format views retired; `Block`
  non-exhaustive with `Block::Menu`; gopher lowered to it; the feed's dates,
  bodies and enclosures apart; an entry's body as its own document; soft
  breaks a setting). R6, gopher-protocol 0.2.0, is prepared in the smolweb
  repository (`b0f1dda`) and waits on Mark's sign-off to publish. The
  fidelity plan's progress log carries the gates, readings and gaps.
- **2026-10-06.** Round 26: S76 (publish gopher-protocol 0.2.0, then repin) and S77 (knot-editor's preview change noted in its plan, its lane to make it).
- **2026-10-06.** S45 carried out, under rounds 27 and 28 (S78 to S82): Frozen evidence ledgered (`362c5d5a`), the header tool re-run over 75 owned sources in one commit (`43353861`), and the gate added to CI and the doc audit (`0b374437`). The receipt is in the license sweep plan's §6.
- **2026-10-06.** Round 16: S46 (live plans handed to their lanes), S47
  (reviewed write lanes) and S48 (ownerless tails to the archived plan tails
  backlog). S44's repins and S45's header re-run were flagged as separate
  follow-on tasks for Mark to start.
- **2026-10-06.** The S14 pass, phase B done: batches 39 to 51 re-judge
  all 145 plans at `535bca11`, superseding their earlier records (S34). 1,656
  claims checked: 1,190 hold, 408 stale, 58 unverifiable; 241 contradictions;
  85 status lines wrong (59%). Dispositions: 82 current, 34
  historical-marked, 25 historical-unmarked, 4 superseded. Every batch was
  drafted by one read-only subagent and checked by a second; across thirteen
  verifications none refuted a record outright, three refuted single items
  (Graphshell's face picker, athanor's image GC, a surviving
  `is_surface_engine`), and each found about six items true only in part, all
  applied. The largest causes: meerkat's removal (`c5f01064`), the 2026-09-23
  folds, and status lines never updated after Progress moved. The judgment
  audit's one error is another lane's new balaur review brief (`ed8b67e8`),
  committed without a record. Phase C assessed: 166 documents carry
  recommended actions (92 wrong status lines, 440 stale claims); about 67
  are archive candidates under S36; DOC_README has dozens of stale entries
  and 10 orphans; the doc audit reports 14 resolved annotations; about 33
  documents carry the dead `crates/graph/subgraph` banner; and 27 of the 166
  were touched by other lanes since 2026-09-29, about a dozen of them heavily
  (physics catalog, dynamics grammar, burn, the one-tree plan, vault lock,
  device pairing, chatelaine, projection grammar). How phase C runs goes to
  Mark.
- **2026-10-06.** Round 15: S42 (`reference-data` is the lexical plan's
  L2), S43 (the lattice pass continues), S44 (all three `sha2` 0.10 pins
  repinned, as a follow-on) and S45 (headers re-run and gated, as a
  follow-on).
- **2026-10-06.** Phase B, batches 39 to 46 recorded (96 plans; the
  per-batch counts are in each record), each drafted by a read-only subagent
  and checked by an independent read-only verifier whose corrections are
  applied, with a direct sample per batch. No verifier refuted a record
  outright; one claim was half refuted (B7, `is_surface_engine` survives) and
  roughly six items per batch held only in part. Findings past the documents:
  three first-party manifests pin `sha2` 0.10 against the ruled 0.11 row
  (djinn, Pelt, the session fixture); 61 owned sources lack the MPL Exhibit A
  header; merge `0a8198ba` dropped inker's notice files (S40). Round 14: S38
  (forks routed to lanes), S39 (meerkat-era wording), S40 (inker stays
  MPL-2.0) and S41 (S37 ruled directly).
- **2026-10-06.** Round 13: S34 (a newer batch record supersedes the
  snapshot's), S35 (the three receipts renamed, S10's flattened), S36 (plans
  found complete are extracted and archived) and S37 (`reference-data`'s lane
  asked first). Phase B's first wave (batches 1 to 4, 48 plans) returned;
  the second (5 to 8) is running.
- **2026-10-05.** The S14 pass, phase A done: D2 records for the 21
  documents added since the snapshot (batches 35 to 37), coverage 337 of 337.
  Three read-only subagents (opus) drafted them at `26060e88`; every stale
  claim and contradiction was re-checked here before it went into a record.
  604 claims checked, 32 stale, 26 contradictions. Seven status lines are
  wrong: the hagiograph plan (H5 landed in isometry the day the plan was
  written, and three crates consume it), lattice sync (the 2026-09-23
  consumer round, CI and the two-row unused-patch baseline unrecorded),
  lexical capability ("not started" while `reference-data`, `9310518b`,
  serves Knot), ranged fetch (T2 unblocked since 2026-09-23), crate
  consolidation (C4, insigne B to D and chatelaine landed), the pre.4
  allocator diagnosis (repair selected by ruling 508 and accepted, pre.4
  merged at `cec0b3a4`), and micron navigation (P1c, Cambium's scroll request
  and A1's Knot half landed). Among the rest: the tree migration inventory
  predates the physics panel and remote session; the R1 receipt's
  callback-only control cannot fail; the S10 receipt's test breakdown is
  51 + 8, not 56 + 3; the controls receipt never records its 2,000-node
  result (ok, interval p50 2217.3 ms). The doc audit reports 14 annotated
  links that now resolve (it exits 0 without `--fail-on-findings`). The
  judgment audit's one remaining error is the three `RECEIPT.md` basenames.
  Phase B's re-judgments cannot be supplemental batch records: the audit
  rejects a supplement that duplicates a legacy record, and DOC_POLICY keeps
  the snapshot unchanged; where they live goes to Mark.
- **2026-10-05.** S32 built on branch `stack-seams-s32` (`3ecba102`) and
  landed on main at `48c08dee`, main (`2c4eaa1b`, documents only) merged into
  the branch first. `card_issue` refuses a card whose width or height is not a
  finite positive number with an `items.card` issue: `validation_issues`
  carries it, so `compile`, `refresh` and `compile_snapshot` refuse, and the
  relationship compile adds it ahead of its first refusal. No signature
  changed, so the Knot lane's adaptation stands. Receipts:
  `a_degenerate_card_is_a_typed_issue` (0 by 0, a zero height, a negative
  width, a NaN and an infinite side) and
  `a_relationship_compile_refuses_a_degenerate_card` (0 by 0 and NaN), each
  ending with a usable card that compiles; with `card_issue` removed both
  fail. Gates: scenomise 99 passed; graphshell's `projection` tests 35 passed;
  `cargo check --workspace --locked` exits 0. The new lines add no rustfmt
  differences; the three files' existing ones are left for the repository's
  separate formatting sweep. One graphshell test timed out under load (F21).
  Not pushed.
- **2026-10-05.** The S14 pass begun (S33), at audit base `26060e88`.
  Assessment: 337 active documents, 155 of them plans (81 whose status claims
  landed or done, 21 in progress, 13 planned, 36 otherwise worded, 4 with no
  status line); D2 judgment coverage 316 of 337, the 21 without a record all
  added by other lanes since 2026-09-05; three active receipts share the
  basename `RECEIPT.md`, which the judgment audit rejects. The pass runs in
  three phases: A, a D2 record for each of the 21; B, the 155 plans' status
  lines and done-claims re-judged against the tree, most records dating from
  the early-September snapshot; C, corrections as dated annotations with a
  remediation receipt, after the 2026-09-06 one. Read-only subagents (opus)
  draft the records in batches; every stale claim they report is re-checked
  here before a document changes. *Reading, not ruled*: archive moves for
  plans found complete (DOC_POLICY §8) and renaming the three receipts go to
  Mark as forks rather than being made here.
- **2026-10-05.** Round 12: S32 (a degenerate card is a typed compile issue)
  and S33 (the S14 pass next).
- **2026-10-05.** P2 landed on main at `40d7ae5e`, by S27's one merge: main
  (`102aa548`) was merged into the branch, weave resolving the one shared file
  (`cambium-genet-web-host/src/a11y.rs`, G9's `target_of` removal beside P2's
  window-subtree signatures), and every gate rerun on the merged tree (573
  passed, 0 failed, 9 ignored; the four headed receipts; the web host on wasm32,
  now without a warning; `cargo check --workspace --locked`) before main
  fast-forwarded. Not pushed. P2's done-conditions as built: two windows render
  with one core boot counted, a tenant handed the core shares the surfaces'
  device, a suspend and resume boots none, and mere's consumers build and pass.
  Checkpoints C1, C4 and C5 were not reached: Woodshed's surface compiles as
  written (stage 2's review), no genet change was needed, and no single-window
  assertion changed. Open: F19 for Mark; F20's unmeasured cases; two animating
  windows on X11, Wayland and macOS (F18).
- **2026-10-05.** P2 stage 4 built on branch `stack-seams-p2` at `7ae0e36e`,
  with review fixes at `e131273e`. `run_windows` beside `run`: `WindowsInit`
  (state, sheet, resources, launch windows as lens and options),
  `WindowHooks`, `WindowHost`, and the `WinitWindows` event source over
  `MultiHost`; `WinitHost` takes the tree as a defaulted parameter and the
  single-window lifecycle splits into per-window pieces both entries drive;
  events route by `WindowId` to their window's turn; hooks open, close and
  redraw windows through `ctx.runner`; a window whose close policy exits
  closes, and the last one ends the loop. Rasterization is keyed per host
  (F17). Each window paces itself (S26) through a noted request that the idle
  turn serves; on Windows the idle turn draws each window that asked (F18).
  Receipts: `headed_tests::windows` (two windows at 480 by 360 and 720 by 480,
  one core boot, both surfaces and a tenant on one device, a click in A shown
  in B, one frame presented for that click, a forced suspend and resume
  booting nothing, B presenting 0 frames while A presented 8, closing A leaving
  B up) and its control (a third window after the core is forgotten boots a
  second core on its own device; an animating B presents); windowless, a turn
  opens and closes windows, and a sheet one window swaps from a hook reaches
  the other; GPU, a producer moved between windows survives whichever draws
  first, one key in two windows is a duplicate rather than a theft, and a
  closed window's producers retire. A read-only review subagent (opus) of
  stages 3 and 4 found three bugs (a sheet swapped outside the frame hook never
  reached the other windows; a producer moved between windows was retired if
  the source drew first; a wake redrew only the first window) and five risky
  behaviours (a reached window asked through the native window, doubling the
  acting window's frames: 2 per click before, 1 after; hidden windows drawn by
  the idle turn; one key in two windows thrashing; a closed window's producers
  and window-root left behind; nested shadow trees dropped from the whole
  document's view), all fixed at `e131273e`, each fix with a receipt whose
  control was run except the wake, the hidden-window skip and the cascade of
  windows opened from a first frame. The first click count read 2 for an
  instrument reason (the platform's first paint of a newly shown window landed
  inside the count) and settles both windows before the click now. The
  single-window core boots before `init` again, as before stage 4. Gates on
  the branch: cambium, rootstock, the winit host, cambium-winit-a11y,
  mesquite, mere-view and pelt-desktop pass 573, fail 0, ignore 9 with stage
  1's warnings; the four headed receipts pass; the web host checks on wasm32;
  `cargo check --workspace --locked` exits 0. The non-Windows idle-turn branch
  was type-checked on Windows with the cfgs swapped, as no Linux C toolchain
  is installed here. F18 to F20 recorded.
- **2026-10-05.** P2 stage 3 built on branch `stack-seams-p2` at `21b0057f`, not
  merged. `WindowDom` presents a window-root as its document, and the pipeline's
  layout, hit testing, caret, scroll, paint, producer and accessibility reads go
  through it (a single window's root is the document); `HostTree` gains `mount`
  and `drain_mutations`; `Accessibility::sync` takes the window's subtree and
  `cambium-winit-a11y`'s projection is generic over `LayoutDom`; `MultiHost`
  holds the runner, the forest document and the windows' hosts, and a window's
  turn lends it the runner (its `ctx.runner`, a `WindowTree`), the shared part
  and the hooks; a mutation router files each drained mutation under the
  window it touched, or every window when it cannot place it; each window
  records its leaf keys and the producer registry leaves keys other windows
  hold alone; a shared sheet swap carries a generation. In cambium,
  `GenetMultiRunner` gains the per-window calls the pipeline makes, and its
  trees hand message dispatch and focus collection their mount (the two
  findings recorded with stage 2). Receipts, windowless in rootstock: each
  window lays out its own subtree at its own size and scale; a click in one
  changes the other in the same pass; a change only one window shows rebuilds
  only that window's layout (a change only the other shows is the control);
  each window's accessibility tree, through the host's own sync, is its own
  subtree; a node moved between window roots by `move_before` keeps its
  `NodeId`, is laid out by its new window, and its leaf keeps its painter; a
  producer another window holds is not retired (GPU test, with the retiring
  control). Controls on the instrument: every mount made the document fails
  four receipts, and every mutation filed to every window fails the routing
  receipt. The first accessibility receipt built its own view and passed under
  the scoping control, so it was rewritten to read what the host's sync path
  hands a recording bridge. Gates: cambium, rootstock, the winit host,
  cambium-winit-a11y, mesquite, mere-view and pelt-desktop pass 567, fail 0,
  ignore 7, with stage 1's warnings; the headed receipt and its control pass;
  the web host checks on wasm32. A first gate run failed one test (`RefCell
  already borrowed`): a document borrow stage 3 had bound to a variable
  outlived the caret move and met the update after it; scoped to its block.
  *As built against §3.1's text*: accessibility ids are not salted by window.
  They are opaque ids of nodes in one arena and each window's tree holds only
  its own subtree, so no id can be in two trees (the receipt checks exactly
  that); a salt would only matter if two windows' trees were merged into one
  namespace, which nothing does. The message-dispatch mount fix has no receipt
  of its own; it makes dispatch match what rebuild already did.
- **2026-10-05.** P2 stage 2 built on branch `stack-seams-p2` at `03f8fe74`, not
  merged, no behaviour change. `HostTree`, a sealed trait over the runner calls
  the pipeline makes, implemented for `Runner`; `Host`, `HostState`, `AppCtx`,
  `HostHooks` and four hook aliases take the tree as a fourth type parameter
  defaulted to `Runner`; `AppShared` holds the eight shared fields as
  `HostState::shared`. Gates: the stage 1 suites pass with the same counts (316
  passed, 0 failed, 6 ignored) and warnings; the headed receipt and its control
  pass; the web host checks on wasm32. A read-only review subagent (opus) read
  the diff and every use in Woodshed, Hocket and Knot: no behaviour change once
  the shared fields were declared where they had been (drop order), and no
  consumer that fails to compile. *As built against §3.1's text*, recorded
  rather than ruled because none changes what an application sees: the trait is
  public and sealed, not crate-internal, because it bounds public types; window
  commands and the wake flag stay per window (a window verb names a window; the
  wake's home is stage 4's); and "signatures unchanged" holds as source
  compatibility, not literally, since `AppCtx` and `HostHooks` gained the
  defaulted parameter: a name written with three parameters denotes the type it
  did, and the one pattern that would need an annotation (an untyped
  `HostHooks` literal whose closure calls a method on `ctx.runner`) occurs in no
  consumer. Found for stage 3: the producer registry retires every producer whose
  key is not in the frame's layout (`registry.rs`, `prepare`), which under one
  registry painted per window (S29) would retire another window's producers; and
  cambium's `RunnerTree` builds and rebuilds under its mount but hands message
  dispatch and focus collection the document (`runner.rs`, eight
  `parent: Some(document())` sites and `focusables`).
- **2026-10-05.** P2 stage 1 built on branch `stack-seams-p2` at `76aa1bee`
  (worktree `Code/worktrees/mere-stack-seams-p2`, which took P1's build target;
  P1's worktree removed), not merged. One `Arc<RenderCore>` per host, kept
  across suspend and resume, each surface made from it with its transparency;
  `HostState::render_core` and `AppCtx::render_core` for tenants, set by the web
  host too; the renderer's retained leaf fragments survive a suspend. Receipt
  (`headed_tests::one_core`, headed, Windows): one boot across a forced suspend
  and resume, the same core, a present from the new surface, and the tenant's
  device identical to the surface's. Control (`headed_tests::control`, the core
  forgotten across the suspend): two boots, a different core, the tenant on the
  old device. Gates: rootstock, the winit host, cambium-winit-a11y, mesquite,
  mere-view and pelt-desktop tests pass; the web host checks on wasm32 (one
  warning, G9's unused `MirrorHandle::target_of`, reported to its lane). F16
  and F17 recorded.
- **2026-10-05.** F15: the Knot lane reported that P1 breaks Knot at its next
  build against mere's tree; Mark ruled that lane adapts Knot (burn plan
  §13.46). Told that lane P2 leaves the compiler's surface alone.
- **2026-10-05.** P2 sized against the hosts (round 10 evidence) and S27
  ruled: one staged branch, one merge. Round 11 (S28 to S31): the forest dom,
  one leaf registry painted per window, one pipeline, no check-ins; the four
  stages and checkpoints C4 and C5 written into §3.1 and §4.
- **2026-10-05.** P1 landed on main at `1633be0c`, by Mark's "Merge it, then P2
  (Recommended)": main was merged into the branch and every gate rerun on the
  merged tree (scenograph 8, scenomise 97, graphshell 319 + 5, web wasm32 and
  workspace `--locked` exit 0) before main fast-forwarded. Not pushed.
- **2026-10-05.** P1 implemented on branch `stack-seams-p1` at `dd2cb6fd`
  (worktree `Code/worktrees/mere-stack-seams-p1`), not merged. Done-conditions
  as built: every catalog family compiles from a definition (a test iterates
  the catalog); graphshell's projection tests pass through the shared step, with
  four grid assertions moved to the 180 by 84 pitch S16 rules and the option test
  naming its key; an unregistered id and an unread option are typed issues; no
  written-in cell size, column count or footprint remains; two compilers give
  one score. Controls: grid pitch measured at two card sizes (180 by 84 and 216
  by 116), and a custom solver that compiles only with its registration. Gates:
  scenograph 8 and scenomise 97 passed; graphshell with personal-sync 319
  library and 5 other tests passed, 4 ignored; graphshell web wasm32 check and
  `cargo check --workspace --locked` exit 0; no warnings in changed files. Not
  run: Woodshed and other consumers outside this repository, which meet the
  `ProjectionCompiler` signature at repin (the adoption plan's note says so).
- **2026-10-05.** P1 had stalled: the S18 relay reached the graph-semantics
  lane, not the recipe pass. S19 restarts it with a note left in the adoption
  plan; S20 moves item sizes to the host.
- **2026-10-05.** F13's attribution corrected (the recipe pass is another
  session's, which Mark knows); S18: a message for it relayed through Mark,
  and P1's code waits on the answer.
- **2026-10-05.** F13: the compiler had moved into `scenomise::projection`
  under the projection lane (`c79bb8c2`); S17 rules coordination, then P1 here.
- **2026-10-05.** P1's assessment found the registry holds custom solvers
  only (F12), which invalidated S1's framing; put back as round 4 (S15, S16):
  a built-in catalog beside the registry, and parameters measured from the
  items.
- **2026-10-05.** Round 3 (S11 to S14): P1 is built here in a worktree with
  the whole compile shared over a generic dataset type, P2 follows, and the
  next pass checks plan status against code. Mark asked why the portable
  contract had been avoided; the 2026-07-22 charter was quoted back before the
  question was put again.
- **2026-10-05.** P1's owner settled: this lane, by Mark's answer relayed and
  recorded at `1a2e71db`. S5 and S6 needed no change in the projection grammar
  documents, which already use "arrangement" for positions and "scene" in the
  projection sense.
- **2026-10-05.** S9 done by the identity lane (`b52edea7`); checked here
  against the tree (one `PersonaKey` definition, no `PersonaId` left in mien,
  no outside uses, personae's `PersonaId` unchanged). Its test run (mien's 59
  tests; gemot, moothold and distillery checked) is that lane's report, not
  rerun here.
