# Stack seams plan: catalog, shared device, actions, determinism, two words

**Date:** 2026-10-04
**Status (2026-10-05):** in progress. Thirty-one rulings in eleven rounds
(S1 to S31); P1 landed on main (`1633be0c`); P2 staged (S27 to S31, four
stages in §3.1), stage 1 next; P3 and S7 done as documents; S3 to S6 carried into the dynamics
grammar plan (G8, G9); S9 done by the identity lane (`b52edea7`).

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
  ([spatial compute plan](../technical_architecture/2026-08-13_spatial_compute_plan.md),
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
  [workbench component plan](../../cambium_docs/implementation_strategy/2026-08-31_workbench_component_plan.md)
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
[remote projection host plan](2026-07-22_graphshell_remote_projection_host_plan.md)'s
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

**Ruling S14.** *Where does the next contradiction pass look?* Options: plan
status against code; rulings across plans; sibling repos too; no pass. Mark:
**"Plan status vs code (Recommended)"**. Follows: active plans' status lines
and done-claims are checked against the tree, starting with the documents the
doc audit flags. *Reading, not ruled*: it runs after P1, one lane at a time.

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
