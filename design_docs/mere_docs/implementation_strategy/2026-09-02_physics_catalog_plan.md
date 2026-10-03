# Physics Catalog Plan

**Date:** 2026-09-02
**Status:** in progress (P1 landed 2026-09-02; P1b, P2 on both hosts and P3 the remote board 2026-09-03; the runtime extraction 2026-09-04; P4 web half 2026-09-04, closing with the Graphshell tree port per the 2026-10-01 rulings in §5; P5a-c 2026-10-02: kernel, cell list, lagged seam, setters and the web tree at the third-round web defaults, receipts green, merged; then turnstone and P5d; P7 moved to the [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) 2026-10-02).
**Scope:** A catalog of *distinct physics layout laws* — dynamical systems
over the graph's bodies that produce different layouts because they are
different physics — as a lever beside the arrangement catalog, plus the
composable extra forces the donor's presets were made of, in the canvas
and both Graphshell hosts, extended to the remote board, with receipts.
Not in scope: new integrators (seiche stays rapier), GPU tiers (the spatial
compute plan), the ambient backdrop sims (already a catalog), the donor's
WASM layout mods.

**Related:** [cartography layer brief](../research/2026-05-10_cartography_layer_brief.md)
(§5, the strategy catalogue and the helper-era preset portfolio),
[the cartography–gyre layout seam](../technical_architecture/2026-05-29_cartography_aether_layout_seam.md)
(arrangements compute, physics simulates; the seed/read-back bridge),
[physics scenes and tangibility plan](2026-06-22_physics_scenes_and_tangibility_plan.md)
(the scene and ambient catalogs this one sits beside),
[browser WebRTC carrier plan](2026-08-25_browser_webrtc_carrier_plan.md)
(the web host and the scenario lane the receipts run on).
Donor sources, read 2026-09-02 from the archive (`mark-ik/graphshell`,
branch `webrender-wgpu-branch`, `design_docs/graphshell_docs/implementation_strategy/` *(historical citation)* <!-- doc-audit: historical-path -->):
`canvas/2026-02-24_physics_engine_extensibility_plan.md` (the Layout
Algorithm Reference Table and the Ten Thematic/Topological Physics
Presets), `canvas/layout_algorithm_portfolio_spec.md`,
`canvas/force_layout_and_barnes_hut_spec.md`,
`canvas/layout_behaviors_and_physics_spec.md`,
`system/register/physics_profile_registry_spec.md`.

## 1. The question, and what the code answers today

Mark's framing (2026-09-02): there are catalogs of arrangements, of
node/edge/field/canvas presentations, of scenes — so there should be a
catalog of physics layouts too, an additional lever a scene author reaches
for; and it must not collapse into tuned versions of one force-directed
graph. Verified against the crates:

- **cartography** owns two contracts: `LayoutStrategy` (analytic, one
  shot) and `StreamingLayoutStrategy` (iterative, host-owned serializable
  state). The canvas picker lists eight graph-only analytic arrangements
  (`CANVAS_LAYOUT_STRATEGIES`, `cartography_scene.rs:146`); the seiche
  force-directed default is the host's `None`, unlisted; the pure streaming
  strategies the donor had (`ForceDirected`, `BarnesHut`,
  `SemanticEdgeWeight`) no longer exist in mere's crates — they left with
  `arrangements`.
- **seiche** owns the physics: a rapier world with `forces: Vec<Box<dyn
  Force>>` (`lib.rs:375`) and `add_force` only — no replace, no clear
  (`lib.rs:507`). Seven `Force`s: `NodeExclusion`, `EdgeSpring`, `Boundary`
  (installed at build, `seiche_bridge.rs:58`), `AnchorSpring` (the
  arrangement-as-attractor pull), `AffinitySpring` (semantic affinity),
  `CouplingForce` (fields — the donor's magnetic zones, landed), and
  `BarnesHutRepulsion` (built, exported, never added). A `Force` sees
  `ForceContext { bodies, colliders, joints, bodies_by_node, edges,
  repulsion_solver }` (`lib.rs:343`): positions and topology, no node
  attributes. Tunables: linear damping, pause/play, settle budgets. Two
  catalogs already live in this tier: thirteen declarative scenes
  (`scenes.rs`) and four ambient sims.
- **persistence** (`SavedSceneV1`, `product.rs:213`) carries
  `physics_paused` and `physics_damping`, nothing that names a law.
- **the web host** runs seiche inline through settle budgets (~6 s), same
  fixed force set; the remote board is drawn from the endpoint's score with
  no physics.

## 2. Terms, and the donor's catalog read in them

Three words are in play and they are not the same thing; the donor's docs
use all three, and the plan's first draft conflated the second and third.

- **Layout algorithm** (the graph-drawing literature's word): a procedure
  that computes positions from structure. Two kinds — *analytic*
  (grid, tree/Sugiyama, radial, phyllotaxis, Penrose, L-system, timeline,
  kanban, spectral, embedding) and *energy minimization* (spring-electrical
  à la Fruchterman–Reingold, Kamada–Kawai stress, LinLog/ForceAtlas2,
  annealing). Mere calls the analytic kind **arrangements**; the picker is
  their catalog. The energy kind, run to convergence, is what the donor's
  `arrangements` crate did (`ForceDirected`, `BarnesHut`); run live under
  rapier, it is the next thing.
- **Physics law** (this plan's word; the donor's "Dynamic physics"
  category): a dynamical system over bodies — what force each feels, from
  whom, as a function of what — integrated over time. Live FR is one;
  so are n-body gravitation, particle life, flocking, phase
  synchronization, fluids, rigid bodies with collision. This is seiche's
  tier, and the catalog this plan founds.
- **Physics profile / preset** (the donor's word, `physics_profile_registry_spec`):
  a *named parameter set and extra-force composition over one law* —
  `Liquid`, `Gas`, `Solid` were "semantic parameter sets over the
  Fruchterman–Reingold force model". That is the tuning tier; it is real
  and it stays, but it is not the catalog.

So the lever has three parts, and the donor had all three in some form:

1. **Laws** — distinct dynamics. The catalog.
2. **Overlays** — the donor's Level-2 "post-physics extra forces", each a
   seiche `Force` composable onto any law: `DegreeRepulsion` (hubs spread),
   `DomainCluster` (pull toward the centroid of same-site nodes),
   `HubGravity` (gravity scaling with log degree), `DepthGravity` (BFS depth
   drives one axis: roots up, leaves down), `GridSnap` (spring to the
   nearest grid point), `GravityLocus` (pull toward a point, optionally
   oscillating), and the ones mere already has: `CouplingForce` (fields,
   the donor's zones), `AffinitySpring` (semantic clustering),
   `AnchorSpring` (the arrangement's pull).
3. **Tunables** — damping, settle policy, pause/play, anchor strength,
   overlay strengths. A **profile** is a saved (law, overlays, tunables)
   triple with a name — the donor's ten presets are profiles, and they map
   cleanly:

| donor preset | law | overlays |
|---|---|---|
| liquid | Springs | weak locus |
| gas | Springs | none, no anchor |
| solid | Springs | domain cluster + degree repulsion |
| archipelago | Springs | strong domain cluster + degree repulsion |
| constellation | Springs | degree repulsion + hub gravity |
| crystal | Springs | grid snap |
| tide | Springs | oscillating locus (never settles) |
| sediment | Springs | depth gravity |
| magnet | Springs | fields (landed as `CouplingForce`) |
| void | Still | none |

Every one is the same law. That is the collapse Mark refused, and the
donor's own reference table already pointed past it: its "Constraint-Based
/ Elastic (rapier)" and "Semantic Embedding" rows are different physics,
not presets.

### The laws (v1 — all of them, ruled 2026-09-02; plain labels)

| id | label | dynamics | what it reveals | needs |
|---|---|---|---|---|
| `spring.rapier` | Springs | today's law: rigid-body exclusion, Hooke springs on edges, boundary | local structure, no overlap; the interactive default | edges |
| `charge.barnes-hut` | Charge | Coulomb repulsion between all bodies (1/d, Barnes-Hut O(n log n)) + edge springs: the Fruchterman–Reingold shape | evenly spread neighbourhoods, the classic force picture | edges |
| `stress.kamada-kawai` | Stress | every pair a spring whose rest length is graph distance × L | global distance fidelity: paths unroll to true length, far is far | all-pairs shortest paths, cached per topology |
| `energy.linlog` | Energy | attraction ∝ d on edges, repulsion ∝ 1/d overall, degree-weighted (LinLog / ForceAtlas2) | communities as islands, hubs central | edges, degree |
| `orbit.gravity` | Orbit | n-body gravitation, mass by degree, tangential initial velocity, no rest | the graph as a solar system: leaves orbit hubs | degree |
| `kinds.particle-life` | Kinds | particle life: each node has a kind; an asymmetric kind×kind attract/repel matrix (the ambient sim's law over the graph) | sorting, chasing and fleeing by kind; never at rest | a kind per node — the host's choice per scene (relation family, domain, facet), recorded in the saved scene |
| `flock.boids` | Flock | separation / alignment / cohesion; edge-neighbours are flockmates | constellations that move as groups | edges |
| `sync.kuramoto` | Sync | phase oscillators coupled along edges; angle = phase, radius = distance from focus | communities as phase clusters on a ring | edges, a focus |
| `flow.magnetic` | Flow | directed edges align to a field direction (magnetic springs); with depth gravity, the donor's sediment made a law | hierarchy and direction by physics, Sugiyama's reading without its layers | directed edges |
| `anneal.davidson-harel` | Anneal | stochastic descent on a general energy under a temperature schedule | a balanced settled picture; stochastic | edges |
| `still.default` | Still | no forces; positions are the arrangement's (the donor's void) | the arrangement alone | — |

### The overlays (v1, from the donor's Level 2)

`DegreeRepulsion`, `DomainCluster`, `HubGravity`, `DepthGravity`,
`GridSnap`, `GravityLocus` (with an optional oscillation, the donor's
tide) — six new seiche `Force`s, each composable onto any law; plus the
three already landed. Overlays are what a profile toggles; the donor's ten
presets become the first ten profiles, expressed as (law, overlays,
tunables) and named as they were — the evocative names are the product
tier and belong to profiles, not laws.

In code: seiche gains `set_forces` / `clear_forces`; laws and overlays are
seiche `Force`s (seiche is the published home, so mer3ly's live seiche
path can pick them up); the canvas owns the catalogs
(`CANVAS_PHYSICS_LAWS`, `CANVAS_PHYSICS_OVERLAYS`, `CANVAS_PHYSICS_PROFILES`,
mirroring `CANVAS_LAYOUT_STRATEGIES`), builds forces from graph attributes
(degree, kind, depth, domain, the distance table), and switches the live
simulation — inline or actor — with `Canvas::set_physics_law(id)`,
`set_physics_overlays(ids)`, `apply_physics_profile(id)`.

## 3. Phases

**P1 — laws and overlays in seiche, the catalogs in the canvas.** seiche
gains `set_forces` / `clear_forces` (additive) and the `Force`s: laws
`StressSpring`, `LinLogForce`, `Gravity` (n-body), `ParticleLife`,
`Boids`, `Kuramoto`, `MagneticSpring`, `Anneal`; overlays
`DegreeRepulsion`, `DomainCluster`, `HubGravity`, `DepthGravity`,
`GridSnap`, `GravityLocus`. Each law carries a unit test stating what it
reveals — Stress: a 4-node path settles with end-to-end distance ≈ 3L;
Energy: two cliques joined by one edge sit further apart than under
Springs; Orbit and Kinds and Flock: kinetic energy above a floor after
600 ticks; Kinds: intra-kind distance below inter-kind; Sync: two
communities end in two phase clusters; Flow: a directed chain ends
monotone along the field; Anneal: energy after the schedule below energy
before. Each overlay carries one (degree repulsion spreads hubs; depth
gravity orders a tree top-down; grid snap lands on grid points). The
canvas gains `PhysicsLaw` / `PhysicsOverlay` / `PhysicsProfile` values,
the three catalogs, the setters, the `PhysicsCommand::SetForces` mirror
for the actor, attribute builders (degree, kind by the host's choice,
depth from a focus, domain from URL, the distance table, rebuilt on
topology change as couplings are), and the persisted `physics_law`,
`physics_overlays`, `physics_kind_source` fields on `SavedSceneV1`
(optional, defaults `spring.rapier`, none, `relation-family`). Field
couplings stay under every law; the anchor pull is a tunable. *Done when*
a law or overlay switch on a live canvas replaces the force set without
moving a body until the next tick, every combination round-trips through
the saved scene, the law and overlay tests are green, and a catalog test
proves every id builds and every label is plain.

*Landed 2026-09-02.* seiche: `set_forces` / `clear_forces` /
`velocity_of` / `kinetic_energy`; `laws/` (`StressSpring` +
`graph_distances`, `LinLogForce`, `Gravity`, `ParticleLife`, `Boids`,
`Kuramoto`, `MagneticSpring`, `Anneal`) and `overlays/`
(`DegreeRepulsion`, `DomainCluster`, `HubGravity`, `DepthGravity`,
`GridSnap`, `GravityLocus` with `tidal`), one test each stating what it
reveals; 73 seiche tests green. Canvas: `physics_catalog.rs` with
`PhysicsLaw` / `PhysicsOverlay` / `PhysicsKindSource` / `PhysicsProfile`,
the three catalogs plus `CANVAS_PHYSICS_KIND_SOURCES`, the setters
(`set_physics_law`, `set_physics_overlays`, `toggle_physics_overlay`,
`set_physics_kind_source`, `apply_physics_profile`, `physics_profile_id`),
`LawInputs` (the attribute builders: degree, site groups, cluster groups,
BFS depth, the distance table), `PhysicsCommand::SetForces` for the
actor, and the graph-bound rebuild on `reconcile_derived`; four catalog
tests green (every id round-trips, every law and overlay builds on the
sample graph without moving a body, every profile applies and names
itself back, a living law keeps ticking and Stress survives a topology
change). Graphshell: `SavedSceneV1` gains `physics_law`,
`physics_overlays`, `physics_kind_source` with serde defaults (a legacy
scene opens as Springs); the web host saves them from the canvas and
re-applies them on restore. The web-side restore/save edit
(`web_product.rs`) is wasm-only and could not be compiled this session:
the clean genet worktree the web build reads from
(`worktrees/genet-head`) is gone from disk (see Findings). *2026-09-03:*
the worktree was recreated at the `577e2471e97` pin and the wasm check
passed; P1 is fully verified.

**P1b — the petgraph sources (ruled 2026-09-03, landed the same day).**
The kernel's graph is a petgraph 0.8.3 `StableGraph` behind chartulary,
and petgraph's algorithm shelf maps onto the attributes the laws and
overlays already snapshot, so it joins the catalog as *sources* (where an
attribute is read from — tunables, not laws). The canvas builds a
[`TopologyView`] over the **visible** edges (a directed multigraph, one
edge per relation cell; an undirected simple graph with cost
`1 / multiplicity`), so hidden relations relax the physics as they relax
the springs, and runs: `dsatur_coloring` and `tarjan_scc` for two more
kind sources (**By colouring**: a kind never touches its own kind; **By
island**: each component a kind); `page_rank` as a **mass source** for
Orbit and the hub overlays (`PhysicsMassSource::{Degree, PageRank}`;
seiche's `HubGravity` / `DegreeRepulsion` gain `with_weights`);
`greedy_feedback_arc_set` + `toposort` longest-path layering and
`dominators::simple_fast` as two more **depth sources**
(`PhysicsDepthSource::{Roots, Layers, Focus}`); `min_spanning_tree` as
the **Skeleton** overlay (tree edges stiff, via `StressSpring` at one
hop) with a `skeleton` profile over Charge; and per-node `dijkstra` for
the Stress law's distance table, so a pair joined by three relations sits
at a third of a hop (seiche's `StressSpring::from_weighted_distances`).
`SavedSceneV1` gains `physics_mass_source` and `physics_depth_source`
with defaults. Four more catalog tests (a path two-colours and islands
separate; the hub outranks its linkers and weights average one; layers
survive a cycle and dominators count from the focus, the unreachable node
one below the deepest; the tree takes the thrice-joined pair and Stress
reads it as a third of a hop). seiche 74/74, canvas 192/192, graphshell
scene tests 6/6, wasm check green.

**P2 — the levers in both hosts, with a receipt per law.** The product
panel gains a law picker, overlay toggles and a profile picker beside the
arrangement picker (web: `physics-select`, `overlay-<id>` checkboxes,
`profile-select`; native: the same commands); the chrome hint reads
`<arrangement> · <law>`; the saved scene carries all three. The web
snapshot exposes `physics-law`, `physics-overlays`, `physics-energy`
(kinetic energy). Receipts: `physics_<law>.scn` per law with capture pairs
across `settle 60`, asserting the law's own signature (Stress: two nodes
three edges apart end further than adjacent; Orbit/Kinds/Flock: energy
above a floor at the end; Charge/Energy: no overlapping bodies; Springs
and Still: at rest), and one `physics_profiles.scn` applying each of the
ten donor profiles and asserting its overlays are the ones installed.
*Done when* eleven law receipts and the profile receipt are green and the
captures show what the labels say.

*Landed 2026-09-03 (web half).* The panel gains `physics-select`, an
`overlay-<id>` checkbox per overlay, `kind-source-select` /
`mass-source-select` / `depth-source-select`, `profile-select`, and the
`apply-physics` / `apply-profile` commands; the selects are filled from
the canvas catalogs the first time the panel is seen empty, and follow the
canvas after a profile or a reopened scene. The arrangement picker gains
**Free (physics alone)**, the canvas's `None`. The chrome hint reads
`<arrangement> · <law> · physics <state>`. The snapshot exposes
`physics-law`, `physics-overlays`, `physics-profile` (`custom` when no
profile names the pair), the three sources, `physics-paused` (now the
canvas's own state), and the layout's signature numbers from
`Canvas::layout_stats` — `physics-energy`, `layout-spread`,
`layout-overlaps`, `layout-stretch`. The scenario lane gains `select
<css> <value>` and `check <css> on|off`. Twelve receipts green
(`Code/testing/mere/physics_p2_receipt.md`), and five defects found by
them fixed: the pause toggle, the anchors under every law, Orbit under
damping, Charge's calibration, Still's explosion (see Findings). The
native half is unbuilt: no native surface carries an arrangement picker
today, so the open decision below is where the first one goes.

**P3 — physics on the remote board.** The web host seeds seiche bodies
from the remote scene's items — the projection's score is the seed and,
through the anchor pull, an attractor — and runs the chosen law over them,
reading positions back for drawing (the seam doc's seed/read pair, on the
remote side). A card appended by the host enters with a settle burst.
Host interaction policy; the endpoint's score is never written back.
*Done when* `c4b1_live_board` under Charge shows the second card settle
away from the first (positions and a capture pair) and under Orbit the
cards keep moving.

*Landed 2026-09-03.* The remote board never touched the canvas: it was
drawn each frame straight from the score, and the canvas grows only
through its own graph (`visit`, `add_node_at`), offers no slot update
without a re-seed, and no bare physics step, so a second canvas would
have fought its own seams. The seam is a
new stack piece instead: `mere::canvas::PhysicsBoard` — a seiche
simulation with one body per scene item (keyed by **instance**, because
the fixture's appended cards all share one source id), the score's
positions as anchor slots (`AnchorSpring` at `DEFAULT_BOARD_PULL`, a
twenty-fourth of the canvas's stiffness, so the score is a seed and a
soft attractor and the law is what shows; seiche's `Boundary` at 0.08 is
weaker still, so the default Springs picture stays the score's), and the
law, overlays and sources built through the same attribute builders
(`LawInputs::from_parts`, now graph-free: nodes, edges, a site per node).
`sync` reconciles bodies to the acknowledged scene — a new item spawns at
its slot with a settle burst, the others keep their simulated positions,
every slot re-anchors — `tick` runs while settling or under a living law,
`position` / `energy` / `gap` read back. The web host holds one board,
mirrors its canvas's physics choice onto it every frame, syncs it when
the acknowledged revision moves, ticks it while the remote session is
shown, and draws each card at the board's position (the score's when the
board has not seen it). The snapshot exposes `remote-physics-law`,
`remote-energy`, `remote-gap`. The score is read, never written.
Receipt `physics_remote_board.scn` (over the WebRTC fixture): Charge
after the append ends the pair 210 units apart against 140-unit slots
(`remote-gap >= 170`), Orbit holds `remote-energy >= 1` at two and seven
seconds; `RESULT ok`, 37 steps, 828 frames, three captures. Two board
unit tests (slots held, a new item joins without re-seeding, a departed
one drops; Charge separates overlapping slots, Orbit never rests).

**P4 — drag and add.** The web pointer path reaches `pointer_down/up`;
verify drag pins and re-settles under each law, and adding a node fans it
out and settles (or joins the motion under the restless laws); one receipt
row per law; the donor's reheat-on-structural-change and
place-near-parent contracts (`layout_behaviors_and_physics_spec.md` §2)
adopted as the add behaviour. *Done when* the rows are green on the web
and the native host shows the same by hand.

*Web half landed 2026-09-04.* Both donor contracts were already in the
canvas — `mint_node_at` seeds, reconciles and reheats; `mint_node_as`
seeds a new node beside its origin — so P4 was instrumentation and proof,
not new physics. The canvas gained two reads a pointer-driving host had
no way to ask for: `screen_position_of` / `focused_screen_position` (the
inverse of the crate-private `screen_to_world`) and `dragging_node`. The
web host gained the three-step gesture verbs (`press-focused`, `move-by`,
`release-at`), an `add-node <x> <y> <url>` verb, and three snapshot
fields (`data-dragging`, `data-drag-return`, `data-canvas-nodes`).
`physics_drag` and `physics_add` are green over all eleven laws.

**P5 — repulsion on the host's GPU (proposed 2026-10-01, from §5's rulings
of the same day; approved as written by Mark 2026-10-02, with P6: "Approve
as written" for both; order "P5a–c and P6a in parallel, then P6b"; turnstone
wired by "the P5 lane, after mere lands").** `NodeExclusion` (eight
laws) gains a GPU path on the host's own device, with rapier keeping every
other role and the CPU scan as the fallback.
- *P5a, the kernel.* `NodeExclusion`'s exact law (inverse square,
  `min_distance` floor, `cutoff`) as a CubeCL kernel in `conatus::resident`
  beside `repulse` (`kernels.rs:47`, which is softened and cutoff-free, so
  not reused as is): tiled all-pairs below a node threshold, a cell list
  above it (radix sort and binning ported by hand from Nexus, per the
  licensing ruling). Built through `ResidentClient::init(WgpuSetup)` from the
  host's netrender `WgpuHandles`, never a device of its own.
- *P5b, the seam.* Seiche's `RepulsionSolver` gains a lagged mode: positions
  submitted at step k, forces applied at step k+1, readback non-blocking on
  every host. No adapter, a failed submit or a late result falls back to the
  CPU scan for that step. The synchronous closure stays for tests.
- *P5c, the hosts.* Pictograph's `Canvas` and `PhysicsBoard` take an optional
  device; the web tree passes `ProducerContext.core`'s handles, turnstone
  its own. The actor path installs the solver before offload. The threshold
  is measured per host (the unrecorded `settle_timing_naive_vs_gpu_solver`
  bench, `seiche/src/tests.rs:149`), configurable.
- *P5d, resident mode (after P5a–c).* Measure lagged against resident
  (integration on the GPU, rapier's contacts, joints and materials dropped)
  at 2,000, 10,000 and 50,000 nodes on both hosts, and bring Mark the numbers
  for "resident above a threshold" or "resident by default where a device
  exists".
*P5 rulings, 2026-10-02* (after P5a's tiled kernel, `438187cb` on
`gpu-repulsion`, matched the CPU law to 2.9e-6 at 1k and 1.2e-5 at 10k and
ran 1.58 ms against 212 ms at 10k):
- **Staleness.** A browser readback resolves only between JS turns, and the
  web tree runs up to three physics steps a frame, so the literal one-step
  rule would put about one step in three on the GPU. Mark chose "Newest
  result for up to N steps": the newest completed result applies for up to N
  steps (configurable; 1 native, 3 web), the CPU scan only past N. This
  amends the one-step lag above. The alternatives were the literal rule, the
  lag counted in frames, or one step per frame while the GPU is on.
- **The seam.** "LaggedRepulsion trait beside the closure": submit/poll,
  the Simulation holding either, `ForceContext`'s solver and threshold
  becoming one `repulsion` field, `Force` unchanged, `NodeExclusion` checking
  body order and applying the staleness rule, in a seiche `gpu` feature. The
  alternatives were the lane inside `NodeExclusion`, or a general deferred
  force.
- **The cell list.** Mark chose "Port GPU binning now" against the
  recommendation (CPU binning for the lagged mode, GPU binning only for the
  resident mode): Nexus's count/scan/scatter binning is ported now, with a
  `LICENSES.md` entry. The alternatives were that split, or deferring.
- **The device.** "Setters with a shared PhysicsDevice":
  `Canvas`/`PhysicsBoard::set_physics_device` take a cheap clone the host
  builds once from its `WgpuHandles`; a `PhysicsCommand` delivers it to
  offloaded physics. The alternatives were setters taking `&WgpuHandles`
  (one CubeCL server per canvas), or a constructor argument.
- **Feature gating.** "mere `canvas-gpu` feature": `mere` gains `canvas-gpu`,
  chaining to pictograph `gpu`, seiche `gpu` and `conatus[resident]`, opt-in
  per consumer. The alternatives were graphshell-web naming pictograph
  directly, or GPU on by default under `canvas`.
- **Threshold.** "Node count per host, measured": configurable per host,
  each host's default its measured lagged-mode crossover, 0 meaning always
  GPU. The alternatives were a pair-count or density threshold, or
  calibration at install.

*Open, 2026-10-02: more than one layout at once.* Mark asked: "consider the
situation where more than one physics layout is active... for example, a
barnes hut layout but then burn tensors determining semantic grouping and
expressing that with physics too." Today a Canvas composes one law plus
overlays, with separate coupling, affinity (an `AffinitySpring` from
scores, `strategy.rs:534-616`) and anchor slots. An assessment of the
composition space (weighted, partitioned, multi-integrator and sequenced
laws; Burn-computed semantic grouping as force) comes back to him before
P6's composition fork (pure Density or Density with edges) is put.
*Assessed and ruled, 2026-10-02.* Findings: forces sum through rapier's
`add_force` in `Simulation::tick` (`seiche/src/lib.rs:723-775`: law and
overlays, then coupling, affinity and anchor) with no per-law weights; three
laws write state instead of adding force (Hold, Anneal, Density's CPU tier);
ESP's Burn `affinity_pairs_over_index` (`esp/src/embed/index_burn.rs:144`)
emits exactly the triples the affinity slot takes, but nothing calls
`set_content_affinity` (`strategy.rs:520`) outside tests, and affinity is not
in `PhysicsChoice`; Density with `EdgeSpring` read −0.54 against 0.77 alone
(*annotation 2026-10-02:* this figure came from the Density lane's
first-round probe, whose log was overwritten by its second-round runs, so
no log now backs it; the second-round overlay probe, `probe-overlays.log`,
shows the same failure class for force overlays on Density)
(316 overlaps).
- **Currencies.** "Laws declare currency; catalog adapts or refuses":
  forces compose freely; a kinematic law takes forces converted to its
  currency (overdamped, v = F/γ); a resident law takes only forces with
  resident kernels or the lagged upload; the picker greys out the rest with
  the reason. The alternatives were forbidding every mix with a non-force
  law, or allowing anything.
- **Composition tier.** Mark chose three: "Grouping overlays: semantic +
  partitioned" (a `Grouped` outer law on group centroids with an inner law
  per group, and a semantic overlay, each with a source and weight, saved in
  `PhysicsChoice`), "Weighted multi-law lists" (`law` becomes a weighted
  list, strengths first brought to a common scale), and "Sequenced blends"
  (profiles with a schedule, and capturing positions as anchors). The
  alternative left composition as is.
- **Meaning.** "'Meaning' as a source": beside site and cluster, feeding
  affinity pairs, `DomainCluster` groups, Kinds' kinds and the partition for
  grouped laws. The alternatives were affinity pairs only, or a Semantic law.
- **Density with edges.** "Pure Density now, Bonds later": edges may join as
  a "Bonds" overlay in Density's currency, accepted only if rank stays at
  least 0.8. The alternatives were today's `EdgeSpring`, or holding the fork.
Open: the embedding device and cadence, and whether a per-step semantic
field exists; then the composition work becomes its own phase.
- **Embeddings.** "Burn on the host device, off-path": ESP's
  `bert::load_wgpu` (`provider.rs:544-550`), which boots its own device, is
  fixed to take the host's; embeddings are computed on content change in an
  actor or async task; lexical embeddings on the CPU are the fallback and the
  wasm default until ESP's wasm build is verified. The alternatives were CPU
  only, or keeping `load_wgpu`'s own device.
- **No per-step semantic field.** "Snapshots only for now": meaning enters
  as pairs, groups and kinds snapshots through the existing rebuild path, so
  the `LaggedRepulsion` seam ruling stands. The alternatives were
  generalising it to deferred forces, or a second trait when a field law
  exists.
The composition work is P7 below.

*P6a rulings, 2026-10-02* (after `3e477143` on `density-cpu`: the CPU tier
with re-splat flow, walls, Voronoi area share; a 12-node sample at Spearman
0.83 to 0.88, Springs 0.07, the gradient fault failing every receipt):
- **The field.** Mark chose "Gastner–Newman evolving field", against the
  recommendation (re-splat every tick, a live law reacting to drags, which
  plateaued near 0.82 with degree mass): splat once, diffuse, nodes ride the
  evolving field, made stable for point masses by adaptive substeps (the
  prototype gave 78 to 1,153 overlaps and rank at most 0.55 at frame-rate
  steps). *Reading, not ruled:* how a drag or an added node re-enters the
  field returns as a fork if the lane finds more than one way.
- **Boundary.** "Walls": a box sized to the target area. The alternative
  was a sea following the graph (0.32 against 0.77).
- **Area share.** "Raster Voronoi, bbox + margin", positions only, readable
  for any law. The alternatives were clipping to Density's walls, or a
  smoothed volume.
- **The bar.** "Bar on degree mass, PageRank recorded": the receipt asserts
  Spearman ≥ 0.8 with degree mass; PageRank's figure is recorded. The
  alternatives were 0.8 for both, or a lower PageRank bar.
Open: the settle budget and defaults (they move with the field choice), and
the composition with edges (waiting on the composition assessment).
*Second round, ruled 2026-10-02* (after `746b6902` on `density-cpu`: the
evolving field with CFL substeps is stable, 0 to 10 overlaps against
Springs' 38 to 173, but one pass settles at Spearman 0.60 on the 200-node
graph; repeated passes reach 0.81 at 60 one-second passes and bring uniform
mass to CV 0.05; at the catalog defaults overlays mixed in silently, Hub
pull reading −0.33 with 510 overlaps):
- **Re-entry.** "Repeated passes": each pass re-splats where the nodes are,
  absorbing drags, contacts and additions, and a structural change still
  starts a fresh flow. The alternatives were restarting on drag release and
  structural change, or a local re-splat or field blend.
- **Settle.** Mark chose "Stop on convergence", against the recommendation
  (a per-law budget of passes times seconds): passes end when a convergence
  test says the layout has stopped changing. *Reading, not ruled:* the test
  and its threshold come from measurement and return to Mark if more than one
  is defensible. The alternatives were that budget, or never resting.
- **Defaults.** "64², 1-s passes", blur 0.25: the same ranks as 128² at about
  a sixteenth of the cost. The alternatives were 128² as planned, or 128²
  with fewer sweeps.
- **Overlays on Density.** "Refuse overlays on Density now": the catalog
  refuses them with a reason until the currency work. The alternatives were
  dropping them silently, or allowing and marking the mix.

*Third round, ruled 2026-10-02* (P5 checkpoint `60a990a5` and Density
`5f529c64`):
- **P5 web staleness, reopened.** Chrome answers a readback about two frames
  after submit, so the newest answer was six steps old at every size from 128
  to 2,000 nodes; at the ruled N = 3 the 2,000-node web run used the GPU for
  9 of 418 steps (physics 94 ms a frame against the CPU's 82), at N = 9 for
  413 of 418 (6.1 ms) within the CPU twin's bounds. Mark chose "Web N = 9,
  web threshold 400", amending the staleness ruling's web default. The
  alternatives were deriving N from the step cap, or keeping N = 3.
- **P5's Cambium change.** cubecl-wgpu 0.11-pre.2 always enables wgpu's
  `fragile-send-sync-non-atomic-wasm`, which feature unification puts on the
  page's one wgpu, so wasm `map_async` callbacks must be `Send`. Mark chose
  "Commit Arc<AtomicBool>" for the readback flag in
  `cambium-genet-web-host/src/capture.rs`. The alternatives were vendoring
  cubecl-wgpu without the feature, or keeping `canvas-gpu` off on the web.
- **Density's stop test.** "Shift < 0.05 for 3 passes": passes end when the
  mean node shift stays under 0.05 spacings for three passes, the 120-pass
  cap the fallback, a drag re-arming it (the sample stops at pass 5 at 0.84,
  gen-50 at 6 at 0.78, the 200-node graph at 18 at 0.76). The alternatives
  were a field-CV test, or the cap alone.
- **Density's bar at 200 nodes.** Over 90 one-second passes at 64² the rank
  wanders 0.73 to 0.79; round two's 0.81 was one sample of that trace. Mark
  chose "Accept a plateau bar for large graphs": 0.8 holds on the sample and
  gen-50; the 200-node receipt asserts at least 0.7 with its plateau
  recorded, revisited with P6b's 512² GPU grid. The alternatives were
  searching for other defaults, or changing the law.

*Done when:* GPU and CPU forces agree to 1e-3 relative at 1k and 10k nodes;
a sign-flipped kernel fails the overlap check (positive control); a forced
adapter failure falls back and still passes; the eleven law receipts stay
green at the default threshold; a 2,000-node settle at threshold 0 passes the
same energy, overlap and spread bounds as the CPU path on the web tree and in
turnstone, with the call count proving the GPU ran; physics cost per frame is
recorded beside the CPU figures; and P5d's numbers are in front of Mark.

**P6 — Density, the first GPU-tier law (proposed 2026-10-01, same
standing).** `density.gastner-newman`, label "Density": nodes advect along
`-∇ρ/ρ` of a diffused density grid until area follows mass (the mass source
is the existing one, PageRank or degree).
- *P6a, the CPU tier.* `seiche/laws/density.rs`: splat mass onto a grid,
  Jacobi diffusion, bilinear gradient, advection; 128² by default. Appended
  to `PhysicsLaw` and the catalogs; visible on every host.
- *P6b, the GPU tier.* Grid splat, diffusion and gradient kernels in
  `conatus::resident` (fixed-point atomics or gather for the splat, MPM's
  particle↔grid transfer ported by hand), and `integrate` (`kernels.rs:177`)
  gains a kinematic mask so drag pins hold. Nodes run resident under this law
  (ruled), with readback through P5b's non-blocking path; 512² by default.
- *P6c, receipts.* `physics_density` on the old page and the tree, native in
  turnstone, and the GPU tier on both hosts after P5.
*Done when:* the GPU grid matches the CPU reference field to 1e-3 mean
relative error; settled area share correlates with mass at Spearman ≥ 0.8
(a new `LayoutStats` field) and uniform mass gives a low coefficient of
variation of density; Springs fails that correlation (negative control) and
removing the gradient term fails it too (fault injection); a dragged node
holds under the GPU tier; n = 1,000 at 512² steps within 16 ms resident on
this machine; the CPU tier passes at n = 200 and 128² on the web; and the
eleven existing receipts stay green.

**P7 — composition: several layouts at once (proposed 2026-10-02 from the
same day's rulings; held the same day, see below).**
*Held for a dynamics grammar (2026-10-02).* Mark said: "in the way we've
managed to describe projection grammar, we should probably think about an
overarching model of physics; combinatorial, algorithmically diverse...
more thoughts?" Told that the projection grammar's move (factored
dimensions, a portable artifact, ensure/encourage satisfaction, effectiveness
knowledge beside the grammar) maps onto physics as terms (interaction
topology, kernel, state moved), sources as channels, scope as selection,
combinators (sum, scope, sequence, level-of-detail condition, constraint),
currencies as the type system, potential against non-conservative terms, and
observables as receipts, and that this would subsume P7, he ruled: "Research
brief first, P7 held" (a prior-art shelf with one-line transfers, and every
current law, overlay and slot decomposed, before P7 is rewritten from it;
the alternatives were the brief alongside P7a, or extending P7 directly);
"Its own doc beside projection grammar" for the grammar's plan (the
alternatives were inside this plan, or inside the projection grammar plan);
and on treating arrangements and laws as two realizations of one objective
model, "Yes, as a hypothesis the brief tests" (the alternatives were adopting
it now, or keeping them separate). The rulings above on currencies, the
composition tier, meaning as a source, embeddings and snapshots stand as
inputs to the brief. *Brief written 2026-10-02:* the
[dynamics grammar brief](../research/2026-10-02_dynamics_grammar_brief.md)
(its §11 holds the forks, including a draft rewrite of P7).
*The brief's first forks, ruled 2026-10-02.* The hypothesis held for the
energy class only (6 of 12 laws, 5 of 8 overlays, the anchor and affinity
slots, Spectral alone among the arrangements) and failed for dynamics-only
laws (Kinds, Flock, Flow's needle; Orbit and Sync, whose minimisers are
collapse and total synchrony) and for generator arrangements.
- **F1, the model.** "Terms and targets": one specification model of
  energy, dynamics and target terms; arrangements stay `Score`-recorded
  generators referenced as targets; optimizer realizations are gated on
  class and metric. The alternatives were an energy-only grammar with the
  living laws outside it, or separate grammars sharing only sources.
- **F2, P7's home.** "P7 moves into the grammar plan": P7 becomes the
  grammar plan's build tracks (the brief's G1 to G6), and this section closes
  with a pointer once that plan exists; P5 and P6 stay here. The alternatives
  were P7 rewritten here, or P7 as written.
- **F3, first tracks.** "Declarations and instruments first": G1, every term
  declaring topology, kernel, state, currency, class and observable, with an
  energy-descent test and a reciprocity test that agree with each declared
  class and Kinds failing descent as the positive control; then G2 to G6.
  The alternatives were the spec artifact first, or the combinators first.
- **F4, the spec's home.** "Portable shape now, in seiche": a
  `DynamicsSpec` in seiche, sources resolved host-side, `PhysicsChoice` the
  binding, carried in `SavedSceneV1` now and as a shelfmark delta section
  when a citing consumer asks. The alternatives were growing `PhysicsChoice`
  in the canvas, or placing it in sceno beside the `Score`.
The rest, ruled the same day:
- **F5, the common scale.** "Reference-configuration normalization": a
  term's weight-1 strength is its force at a declared reference (contact
  distance 36 for repulsions, as Charge's calibration did; one rest length of
  stretch for springs; one rest length of offset for unary pulls). The
  alternatives were energy normalization, or per-pair calibration.
- **F6, Anneal.** "Annealing becomes a realization": any energy composition
  may settle by annealing; `anneal.davidson-harel` keeps opening as Springs'
  terms under annealing, so saved scenes reopen. The alternatives were
  keeping it a law with an edge-crossing term, or leaving it.
- **F7, Energy's kernel.** "Keep (1, −1), relabel ForceAtlas2": the code's
  ForceAtlas2 model stays, its docs and label are corrected, and the
  attraction exponent becomes a kernel parameter so true LinLog is a tuning;
  the id stays. The alternatives were switching to true LinLog, or both as
  laws.
- **F8, accidental non-conservatism.** "Declare them as they are": Hub
  room, Hub pull and Flow's needle are classed honestly and the class gates
  realizations; revisited when an optimizer rung is built. The alternatives
  were making them gradients now, or offering both forms.
- **F9, the grouped spread rule.** "Weight share": each member takes its
  weight share of its group's centroid force, the true gradient. The
  alternative gave every member the full force.
- **F10, Kinds.** "Measure first": Kinds' energy at 6 s and 30 s under
  continuous ticking on the P2 fixture decides whether it joins
  `never_rests` or the plan's finding is corrected. The alternatives were
  adding it now, or correcting the finding now.
- *P7a, currencies.* Every law declares its currency (force, kinematic,
  resident). The catalog composes forces freely, converts forces into a
  kinematic law's currency (overdamped, v = F/γ, the rule Hold already
  follows), admits into a resident law only forces with resident kernels or
  the lagged upload, and greys out anything else in both pickers with the
  reason. `PhysicsChoice` gains the composition fields below and the
  affinity toggle, so a composition is picked and saved like a law.
- *P7b, meaning as a source.* A "Meaning" source beside site and cluster:
  ESP embeddings (Burn on the host device through a fixed `load_wgpu`,
  lexical on the CPU as fallback and wasm default), computed off-path on
  content change, yielding top-k pairs (`affinity_pairs_over_index`),
  cluster assignments and kinds. It feeds the affinity slot (finally
  wired through `set_content_affinity`), `DomainCluster` groups, Kinds'
  kinds and P7c's partition.
- *P7c, grouped laws.* A `Grouped` composition: an outer law over group
  centroids (seiche's Barnes–Hut is already a pure function over points,
  `barnes_hut.rs:57`), each centroid's force spread over its members, and an
  inner law per group under a membership mask. The first instance is the one
  Mark named: Charge (Barnes–Hut) between meaning clusters, springs within.
- *P7d, weighted law lists.* `law` becomes a weighted list. First, the
  laws' strengths are brought to a common scale (each calibrated alone today,
  e.g. Charge at 6,000), so a weight means the same thing across laws; then
  mixes are allowed within one currency.
- *P7e, sequenced blends.* A profile may carry a schedule of compositions,
  and "capture positions as anchors" freezes one layout as the attractor for
  the next (the anchor slot already does this for the remote board).
*Done when:* every law reports its currency and the pickers refuse an
incompatible mix with its reason (a test per currency pair); a Meaning
source built from real ESP embeddings on a fixture graph with known topics
yields clusters whose purity against the topics is recorded, on native GPU
and on the CPU fallback, and the GPU path shares the host's device (asserted
single device); the grouped Charge-between, springs-within layout separates
meaning clusters (between-cluster gap greater than within-cluster spread,
stated as a ratio) while keeping edge structure inside each cluster (a
within-cluster stress figure no worse than Springs alone), with Springs
alone as the negative control and a shuffled-meaning control failing the
separation; a weighted mix of two force laws at common scale matches each
pure law at weights 1/0 and 0/1; a sequenced blend reproduces the captured
anchor layout within a stated tolerance; compositions save and reopen with
the scene; and the eleven law receipts plus Density's stay green.
*Closed here, 2026-10-02: moved to the
[dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) as its tracks*,
per F2 ("P7 moves into the grammar plan"). The text above stays as history.
The grammar plan carries it:
- P7a's currency declarations go to G1, their enforcement to G3, and its
  `PhysicsChoice` fields and affinity toggle to G4;
- P7b becomes G2;
- P7c, P7d and P7e become G3's Groups, weighted sum and Schedule;
- each done-condition above goes to the track that carries its part.
P5 and P6 stay in this plan.

## 4. Findings

- 2026-09-02: `BarnesHutRepulsion` has been one `add_force` from live since
  2026-06 (`2026-07-03_archived_plan_tails_plan.md:170`); Charge is its
  honest home, because it replaces `NodeExclusion`'s pairwise exclusion
  rather than adding to it — a different physics, not a tuning.
- 2026-09-02: the donor's ten presets are one law with different
  overlays (table above); its reference table's genuinely different physics
  were rapier constraints and semantic embedding, and its
  `force_layout_and_barnes_hut_spec` §5 said in as many words that
  Barnes-Hut "is a scaling implementation choice, not a new user-facing
  semantics model". The donor never had a law catalog; it had a profile
  registry over FR. This plan is the first to separate the two.
- 2026-09-02: a `Force` sees positions and topology but no node
  attributes (`lib.rs:343`); laws and overlays that need degree, kind,
  depth, domain or graph distance take them at construction from the
  canvas, the way `CouplingForce` snapshots its targets, and are rebuilt on
  topology change as couplings are.
- 2026-09-02: `Force` is `&self` and forces run in registration order;
  laws with interior state (Sync's phases, Anneal's temperature, the
  oscillating locus's clock) keep it behind a `Mutex`; a law plus its
  overlays is one ordered `Vec<Box<dyn Force>>` that `set_forces`
  installs in the declared order (law first, overlays after, couplings
  last).
- 2026-09-02: on the web the simulation is inline and ticks only through
  settle budgets (`input.rs:590` is the continuous run the play control
  enters); Orbit, Kinds, Flock, Sync and the oscillating locus never rest,
  so a law declares `wants_continuous_tick`, the rider the physics scenes
  plan added for perpetual scenes. Landed as `PhysicsLaw::never_rests` /
  `PhysicsOverlay::never_rests` on the canvas side: a switch to a living
  law settles for `u32::MAX` (the play control's continuous run) instead
  of the ordinary budget, so no seiche-side rider was needed.
- 2026-09-02 (P1): the node snapshot every law starts from iterated
  `bodies_by_node`, a `HashMap`, so a seeded law (Anneal's walk) drew its
  random numbers in a different node order per process and a seeded run
  did not reproduce. `laws::node_positions` now sorts by node index; every
  law and overlay reads through it.
- 2026-09-02 (P1): rapier weighs every node body the same (density-scaled
  to mass ≈ 1), so Orbit's degree masses must be gravitational only: the
  law applies `m_inertial · G · m_other / d²`, and the one-time kick is
  the circular speed for the mass *inside* each body's radius (sorted,
  prefix-summed), or the outer leaves start unbound.
- 2026-09-02 (P1): `EdgeSpring` holds spokes at 170 with stiffness 10;
  an inverse-square hub push sized safely for close range is under a
  pixel of displacement at that reach, so `DegreeRepulsion` falls off as
  `1/d`. The overlay tests are with/without comparisons on the same graph
  for this reason: an overlay that does nothing measurable is a defect.
- 2026-09-02 (P1): the profile catalog test found Gas ≡ bare Charge,
  Magnet ≡ bare Flow, Void ≡ bare Still; a picker cannot name the live
  choice when two profiles coincide, so the bare-law profiles exist only
  for the eight laws the donor's ten do not already offer bare.
- 2026-09-02 (P1): the web build's genet worktree
  (`C:/Users/mark_/Code/worktrees/genet-head`, the machine-local
  `.cargo/config.toml` redirect) no longer exists — genet's worktree list
  shows only `repos/genet` (main at `76a47850946`, 34 commits past the
  `577e2471e97` buckram pin, mid crate-split and dirty in cambium) and a
  Codex worktree. Recreating it at `577e2471e97` (or wherever the config
  was last proven) is the way back to a wasm build; that is a git action
  in a repo this plan does not own, so it waits for Mark.
- 2026-09-02 (P1): `tests::fold_and_source_time::source_time_canvas_scrubs_every_canvas_arrangement_without_rewriting_live_truth`
  failed once in the full canvas run and passed three times alone; the
  compared bytes are a graph snapshot that carries `timestamp_secs`, so a
  second boundary between the two serializations breaks it. Pre-existing,
  not touched here.
- 2026-09-03 (P1b): the kernel's petgraph is `pub(crate)` inside
  `chartulary::Graph`, and the signals crate reaches it only through a
  two-method `TopologyView` trait (keys + undirected neighbours), so the
  canvas builds its own petgraph view from the visible edge list rather
  than borrowing the kernel's. Linear in the graph, rebuilt per force
  rebuild, and it respects hidden relations, which the kernel's inner
  graph could not.
- 2026-09-03 (P1b): the Focus depth source snapshots the focused node at
  build; a selection change alone does not rebuild the forces (only a
  topology change or a source switch does). Open: whether a focus change
  under the Depth overlay should rebuild — a small P2 item if the receipt
  shows it matters.
- 2026-09-03 (P1b): asserting the same relation kind twice between a pair
  is idempotent in the kernel, so a multiplicity fixture must lay distinct
  kinds; the catalog tests' `wired` helper cycles through four.
- 2026-09-03 (P2): the web host's pause toggle flipped a remembered flag
  that the canvas's own pause (an arrangement pick pauses it) had left
  behind, so the first click paused a paused sim; every first-run law
  switch happened frozen in the spiral seed. The canvas is the authority
  now, for the toggle and the snapshot field both.
- 2026-09-03 (P2): the arrangement's anchor springs act under every law
  (by design — the anchor pull is a tunable), so a law receipt under the
  boot Spiral measures the anchors: Stress stretched 1.45 instead of ~3.5,
  Flow was squeezed into seven overlaps. The web picker had no way to say
  "no arrangement"; it has **Free** now, and the receipts pick it first.
- 2026-09-03 (P2): Orbit died under the host's damping within six
  seconds — gravity conserves energy only in a frictionless world, and the
  seiche test ran too short under too little damping to see it. `Gravity`
  cancels each body's damping with `m·d·v` along its velocity
  (`counter_damping`), and the receipt reads energy at one and six seconds.
- 2026-09-03 (P2): `BarnesHutRepulsion`'s default strength (2 400, `1/d`)
  is a third of `NodeExclusion` at a node diameter, exactly the
  recalibration its own docs deferred; Charge is built at 6 000 in the
  canvas, matched at contact. Two touching pairs became none.
- 2026-09-03 (P2): Still as an *empty* force set exploded (energy 10⁶,
  bodies off the canvas): rapier's contact solver alone met the tight free
  seed and accumulated separating velocity every tick, with no repulsion to
  spread the bodies first as every other law has. Still is the seiche
  `Hold` force now — velocity zeroed each tick — so contacts nudge.
- 2026-09-03 (P2): under Springs with no arrangement the fixture never
  reaches "energy ≤ 5": a probe read the spring ring down in four seconds
  and then a steady outward drift, the two disconnected components pushing
  apart against the weak `Boundary` at a few pixels a second. The trio's
  free equilibrium on a disconnected graph is slow; nobody saw it because
  the canvas has always run under an arrangement. The receipt asserts the
  honest signature (ring-down gone, no overlaps, not flying apart); the
  drift itself is a `Boundary` / exclusion-cutoff tuning left open.
- 2026-09-03 (P2): the driver's exit code reports the driver, not the
  scenario; `result.json`'s `scenario.state` is the receipt.
- 2026-09-03 (P2): `Canvas::layout_stats` runs a BFS from every node per
  frame for `stretch`; trivial on the fixture, `O(n·m)` on a large graph.
  Worth gating on node count if the web host ever carries thousands.
- 2026-09-03 (P2, native): the native Graphshell is **turnstone**; the
  binary called `graphshell_native_host` is the browser extension's
  native-messaging relay, not a windowed app. Turnstone's commands are its
  palette `Action`s (self-drive scenarios fire them by label with `act`),
  its panes are registered definitions over a renderer, and its settings
  pane already renders real choice and toggle controls from cambium's
  `setting_row`. No native surface carried an arrangement picker before
  this; the Arrange pane is the first.
- 2026-09-03 (P2, native): turnstone pins mere by git rev. Between its pin
  (`541f5ad7`) and origin/main (`8bb15d78`) the other lane moved mere's
  genet pin 72 commits (`eff0cb6` → `b78e2b9`): the document lanes, the
  knot editor host, Pelt and the surface API left genet for mere. Pinning
  turnstone at `8bb15d78` therefore splits the genet lineage (cambium,
  workbench, the scripted DOM twice in the graph), and matching its genet
  pin runs into the moved crates — the other lane's migration, not this
  plan's. The way through: the other lane rebased this plan's P2 commit
  onto their work as `ca47d6ef`, four commits *before* the genet bump, so
  turnstone pins mere at `ca47d6ef` — the catalog on the old lineage. The
  catch-up to `b78e2b9` stays with the migration lane.
- 2026-09-03 (P2, native): this plan's mere commits were rebased by the
  other lane (`7f4bb8c7` → `ca47d6ef`, and P1/P1b likewise); the plan's
  earlier hashes name commits that no longer exist on main. Subjects are
  the durable handle.
- 2026-10-02 (P5a): the tiled all-pairs kernel (`conatus::resident::kernels::exclude`)
  matches `node_exclusion_reference` at a worst per-body relative error of
  2.9e-6 (n = 1,000) and 1.2e-5 (n = 10,000), mean 1.6e-7 and 1.8e-7, on a
  scatter at settled density (one body per 140² px, a twin every twentieth body
  inside the floor, most pairs past the cutoff). RTX 4060 Laptop, Vulkan.
- 2026-10-02 (P5a): cost per call on that machine, upload + dispatch +
  blocking readback against the single-threaded CPU law: 500 nodes 0.69 vs
  1.61 ms; 1,000 0.60 vs 7.35; 2,000 1.24 vs 18.3; 5,000 1.02 vs 77; 10,000
  1.58 vs 212; 20,000 3.15 vs 679; 50,000 10.1 vs 3,035. Tiled all-pairs alone
  stays under a frame to 50,000 here; the cell list is a large-n and
  weak-device matter, not a 10,000-node one.
- 2026-10-02 (P5a): CubeCL's readback is `ComputeClient::read_async`, a future
  that is safe to poll with a no-op waker: on native CubeCL's own poll thread
  drives the map (`cubecl-wgpu` `compute/poll.rs`), in a browser the event loop
  does, so a result is ready no earlier than the next JS turn. The blocking
  `read_one` the resident lane uses goes through `read_sync` and is native-only.
  `init_device` from a host's `WgpuSetup` is synchronous on wasm as well; the
  async setup path (`init_setup_async`) is only for a device CubeCL boots.
- 2026-10-02 (P5a): `conatus --features resident` checks for
  `wasm32-unknown-unknown` (burn and rapier3d included). A release wasm probe
  that links the exclusion lane through `ResidentClient` is 2,580,991 bytes; the
  same lane on bare CubeCL is 2,544,749, so Burn is dead-stripped and costs
  build time, not bundle. The graphshell-web graph carries no CubeCL, Burn or
  rapier3d today; its dev bundle was about 72 MB on 2026-09-08.
- 2026-10-02 (P5a): the positive control needs a fixture the CPU law does not
  overlap on its own. A ring seeded by golden angle tangles (482 overlapping
  pairs under the CPU law) and a spring lattice buckles (1-5); unlinked bodies
  under exclusion and the boundary settle with none, and the sign-flipped
  kernel then leaves 520-544.
- 2026-10-02 (P5b): with one submission in flight, an answer that takes d
  steps to arrive is d to 2d-1 steps old while it serves, so N = 3 on the web
  tree's three-step frames would still put one step in three on the GPU. The
  lane keeps up to N submissions in flight, answered in order; every web step
  submits and the first step of a frame gets an answer one step old. Cost: up
  to N device calls a frame. This is how the ruled "newest result for up to N
  steps" is met, not a change to it.
- 2026-10-02 (P5b): CubeCL's poll thread delivers map callbacks late to a
  caller that sleeps between looks: submit-to-ready was 14 ms at the median
  (p90 25-27 ms) against 1 ms for a spinning caller. A non-blocking
  `device.poll(Poll)` on the host's device before each look
  (`ResidentClient::poll_device`) brings it to the second look, about 2 ms.
  Without it a 60 Hz loop got 131 of 600 steps on the device at 200 nodes.
- 2026-10-02 (P5b): dropping a CubeCL read before it finishes releases its
  staging buffer while mapped, and the next submit that reuses it fails wgpu
  validation ("Buffer ... is still mapped"). A law switch, a rebuilt
  simulation or a closed canvas drops in-flight answers, so a dropped
  `PendingExclusion` is now adopted by its `ResidentClient` and polled to the
  end. The receipt fails with the adoption disabled.
- 2026-10-02 (P5b): an unpaced native loop (ticks back to back) outruns the
  readback even with the poll: 200 of 600 steps on the device at 2,000 nodes
  before the poll fix. A native actor paces at 60 Hz, so the receipts pace too.
- 2026-10-02 (P5b): two CPU-only `Simulation`s with the same seed are not
  bit-identical; `NodeExclusion` sums over `bodies_by_node`, a `HashMap` whose
  order is per instance, so positions differ in the sixth significant digit.
  The refused-device receipt compares against the CPU-to-CPU spread.
- 2026-10-02 (P5b, native, F6): lagged-mode busy time per tick at 60 Hz,
  CPU law / device lane, in ms: 100 0.06 / 1.04; 200 0.23 / 1.02; 300 0.66 /
  1.09; 500 1.73 / 1.12; 750 3.74 / 1.98; 1,000 5.67 / 1.39; 2,000 15.2 /
  3.25; 5,000 66.7 / 3.67; 10,000 191 / 6.76. The device lane has about 1 ms
  of fixed cost a tick; the crossover on this machine is about 400 nodes.
- 2026-10-02 (P5c, web): Chrome answers a readback about two frames after the
  submit whatever the frame's length: the newest answer was 6 steps old at
  every size from 128 to 2,000 nodes (three steps a frame, frames 78 ms to
  2 s). At the ruled N = 3 the lane served 1 to 15 steps in 139 to 418; at
  N = 9 it served all but the first five. Physics stage per frame (p50,
  unlinked bodies), CPU / N = 3 / N = 9, in ms: 128 0.3 / 1.2 / 1.3; 256
  1.1 / 1.8 / 3.1; 512 4.5 / 4.0 / 1.5; 1,000 16.1 / 17.8 / 2.1; 2,000
  81.9 / 94.3 / 6.1. So at N = 3 the device never wins on the web and at
  N = 9 it wins from about 400 nodes. Put to Mark (the web N and threshold).
- 2026-10-02 (P5c, web): `cubecl-wgpu` 0.11.0-pre.2 turns on wgpu's
  `fragile-send-sync-non-atomic-wasm` unconditionally; feature unification
  puts it on the page's one wgpu, which then requires a `Send` callback for
  every wasm `map_async`. Three readbacks captured `Rc<Cell<bool>>`:
  `ports/graphshell/src/web_gpu.rs` and `web_timing.rs` (now
  `Arc<AtomicBool>`) and `crates/cambium/cambium-genet-web-host/src/capture.rs`,
  outside this lane's crates (the same change, uncommitted, put to Mark).
- 2026-10-02 (P5c, web): a generated graph with links packs into a jammed ball
  under Springs (its random long edges pull inward): at 2,000 nodes the CPU
  run itself ended with 11,733 overlapping pairs and spread 323, and the end
  energy varied 53k to 767k between runs differing by a handful of steps. The
  spanning tree alone did the same (9,829 overlaps). The settle receipt uses
  `links=none`, unlinked bodies, where Springs is exclusion and the boundary.
- 2026-10-02 (P5c, web): the headed runs found the receipt window occluded
  (`visibilityState` hidden, no frames); this lane's runner copy adds Chrome's
  `CalculateNativeWinOcclusion` disable and the two backgrounding switches.
- 2026-10-02: `~/.cargo/registry/cache` was emptied and recreated at 18:00
  by another process (not this lane); offline web builds then failed for
  missing `.crate` files although the extracted sources remained. This lane
  re-fetched the web graph's locked crates (`cargo fetch --locked`, 772
  crates, lock unchanged).
- 2026-10-02 (pre-existing, seiche runtime): an offloaded simulation under a
  law that never rests holds a `u32::MAX` settle budget, and the actor exits
  on a closed channel only when the budget reaches zero, so dropping such a
  canvas leaves its actor thread ticking. Seen as device answers counted by a
  dropped canvas's actor in the offload receipt (`seiche/src/runtime.rs`,
  `run`: it returns on a closed channel only `if disconnected &&
  ticks_remaining == 0`). Not fixed here.
- 2026-10-02 (P5c): conatus asks for `cubecl = "0.11.0-pre.2"` (and Burn
  `0.22.0-pre.2`) as caret requirements, which admit later pre-releases of
  the same version. The root lock holds pre.2; graphshell-web's lock is
  ignored by convention, so a fresh resolution of it takes the newest
  pre-release it can reach. A clean export of the branch resolved pre.2
  offline only because this machine's cache held no other; online it would
  likely take pre.4, and the root's pre.2 `cubecl-runtime` patch (the wasm
  fix) would go unused. Pinning `=` in conatus, or recording the web lock,
  closes it; open for Mark.
- 2026-10-02: a cargo run without `--locked` re-serializes the root lock,
  swapping the order of the two genet revisions' `fleece` and
  `layout-dom-api` rows; the committed lock passes `--locked` as it stands, so
  lane commands run locked.
- 2026-10-02 (dynamics grammar G1, F10): the 2026-09-02 finding that Kinds
  never rests holds. On the P2 fixture under continuous ticking its kinetic
  energy is about 18 300 at 6 s and 140 500 at 30 s, against the floor of 1,
  and Kinds joined `PhysicsLaw::never_rests`. The receipts' Play preamble
  had already kept every law ticking, since a settle keeps the larger budget
  (the [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md),
  Findings).
- 2026-10-02 (dynamics grammar G1, F7): the laws table's Energy row ("LinLog /
  ForceAtlas2") describes ForceAtlas2's (1, −1) force model; LinLog proper is
  the law's attraction exponent 0, a tuning, and the id stays `energy.linlog`.

## 5. Decisions

Ruled 2026-09-02: laws are distinct dynamics, never tunings of one; all
eleven laws in v1 ("plenty"); labels plain, ids technical; the kind for
Kinds is the host's choice per scene; the catalogs live in the canvas and
the forces in seiche; the DOC_README index line added this session; the
donor's ten preset names return as the first ten profiles. Open: where the
native picker sits (the product panel, or the scene settings page the
physics-scenes plan founded).

Ruled 2026-09-04: the physics backend is a **public stack type in seiche**
(`seiche::runtime::Physics`), not a canvas internal and not a new crate,
with the actor half behind a default feature; and the remote board moved
onto it in the same pass. What this does *not* yet do: nothing calls
`PhysicsBoard::offload`, so a native board still ticks inline until a host
asks for the thread. (The energy gap closed the same day, below.)

Taken in P1, for Mark to confirm or overturn: the kind sources v1 offers
are **by site** (URL host), **by cluster** (the Louvain partition) and
**by degree** (isolated / leaf / connected / hub), default *site* — not
the `relation-family` default P1's text named, because a node's relation
family is a graphshell-tier taxonomy (`RelationFamilyFilter`) the canvas
does not see; adding it means threading the family per node down as an
attribute, which is a small P2 item if wanted. A long tail of sites folds
into eight kinds (particle life reads best with a handful).

Ruled 2026-09-03: the petgraph shelf joins as sources, all four packs
(kinds by colouring + island, mass by PageRank, depth by layers +
dominators, the Skeleton overlay + weighted Stress), landed before the
pickers as P1b so P2 exposes every source in one pass. Not taken:
matching, a Steiner tree over the selection, cliques as groups — no
inference carried.

Ruled 2026-10-01, after a status review found P4's web half green since
2026-09-04 but its native "by hand" condition unrecorded, the native picker's
home still open, and `PhysicsBoard::offload` uncalled:
- **P4 closes after the tree port.** Asked how to close P4, Mark chose
  "Close only after the tree port": the Graphshell one-tree plan's physics
  panel scenarios (§1 there, "Phase 4 physics-panel rulings") are P4's
  close-out, and P4 stays open until they land. The alternatives were a
  recorded native receipt then close, or closing on the web evidence alone.
- **Scale: wire the GPU repulsion hook.** Told that nine of the eleven laws
  carry `NodeExclusion`'s all-pairs scan (`seiche/src/forces.rs:36`, sized in
  its own comment for "dozens–hundreds of nodes"), that Gravity, LinLog and
  Stress are all-pairs as well, that only Charge uses Barnes–Hut, and that
  seiche's `set_repulsion_solver` has no product caller, Mark chose "Wire the
  GPU repulsion hook": hosts turn on the solver above a node threshold. The
  alternatives were measuring each law at size first, routing every
  repulsion through Barnes–Hut, or leaving scale out of scope. *Reading, not
  ruled:* which hosts (native, WebGPU), the threshold, and what the
  all-pairs laws other than `NodeExclusion` do are open for the lane's
  assessment.
  *Correction (2026-10-01, same day):* `NodeExclusion` is in eight laws,
  not nine (Springs, Stress, Energy, Orbit, Kinds, Flock, Sync, Flow;
  `physics_catalog.rs:1012-1071`). The question put to Mark said nine. The
  assessment also found the solver cannot run in a browser (synchronous
  readback panics on wasm), that no native host constructs a Canvas, and
  that its helper boots its own device; the ruling is reopened with Mark on
  that evidence.
  *Reopened (2026-10-01):* Mark answered: "The ingredients individually
  would work in native, web, and mobile… so the cook or the recipe are the
  issue. Consider your (3); no native hosts, why's that?" The premise was
  wrong: turnstone is the native host and builds a `Canvas` per graph
  runtime (`turnstone/src/app/runtime_pool.rs`, `session_lifecycle.rs`),
  with `physics_native.scn`; the assessment searched only mere. The
  remaining findings are recipe faults (own device, synchronous readback,
  `[N,N]` tensors), not limits of the ingredients. The ruling stands open
  until the recipe is put back to him.

Ruled 2026-10-01, the GPU-tier law lane's first forks:
- **What earns a law id.** Asked whether only distinct dynamics get new ids
  or GPU variants do too, Mark said: "I don't mind tunings. Don't present
  alt tunings as alt instruments. You need both novel layout algorithms and
  arrangements/scenes with physics configs of those layout algorithms. So i
  guess 1?" So new law ids are for novel dynamics only; scale versions of
  an existing law stay under its id as a backend tier; tunings are welcome
  as profiles, arrangements and scenes over the laws, never presented as
  laws.
- **The first GPU-tier law.** "Density": a Gastner–Newman density-equalizing
  layout, nodes advecting along the density gradient of a diffused grid
  until area follows mass. The alternatives were Scent (chemotaxis), Current
  (SPH-carried) and Sheet (elastic membrane).
- **The integrator.** Asked hybrid (rapier nodes, GPU medium) or a resident
  GPU backend, Mark asked: "You are allowed to consider nexus. Would that
  help?" Open until Nexus is assessed for it; the conatus plan's "Nexus is
  decomposed, never adopted" ruling is the frame.

Ruled 2026-10-01, after the Nexus assessment. Verified in the checkout
(`crates/nexus` `3bf7c6d`): its rigid-body solver applies gravity only ("no
user forces yet", `src_rbd_shaders/dynamics/solver.rs:290`), it has no
repulsion force, it takes a host's device through `WebGpu::from_device`, its
step reads back without blocking, and linking it adds about 164 crates with
a second rapier/parry. Its MPM particle↔grid transfer, radix sort and cell
binning are the useful patterns.
- **GPU repulsion recipe.** Mark chose "CubeCL on the host device, async":
  `NodeExclusion`'s exact law with its cutoff as a CubeCL kernel on the
  host's `WgpuHandles`, tiled all-pairs below a threshold and a cell list
  above it (ported from Nexus's radix sort and binning), one-frame-lagged
  non-blocking readback, and an `Err` to the CPU path when no adapter or a
  failure. Hosts: turnstone (native) and the web tree; mobile when a host
  exists. This settles the reopened scale ruling above. The alternatives
  were a CPU cutoff grid first, or repairing the Burn hook's recipe.
- **Density's integrator.** "Resident CubeCL now": conatus's CubeCL
  `integrate` gains a kinematic (pin) mask and a grid-gradient force, with
  Nexus MPM's transfer ported by hand. This lifts "seiche stays rapier" for
  the GPU-tier lane only. The alternatives were rapier nodes over a GPU
  grid, or patching Nexus's solver.
- **The Nexus build blocker.** "Fix now": the Windows failure is
  `cargo-gpu-install` removing `Cargo.lock` when spirv-std comes from
  crates.io (`crates/cargo-gpu/crates/cargo-gpu-install/src/install.rs:311`),
  a different step from the version gate the local fork fixed. The
  alternative deferred it until a vessel gate needs GPU dynamic bodies.
- **Licensing.** Mark said: "Doublecheck. Otherwise, 1". Double-checked:
  every Nexus crate declares `MIT OR Apache-2.0`, but the repository ships
  only the Apache-2.0 text and GitHub detects Apache-2.0, unchanged on
  upstream main `1cfbd76` (2026-10-01); its radix sort is copied from brush
  (Apache-2.0); `vortx` is Apache-2.0 only. So Nexus-derived code is treated
  as Apache-2.0: a close port is a retained-license entry in `LICENSES.md`
  with upstream notices, and code merely informed by it stays MPL-2.0 with a
  credit line, as `crates/intel/esp/src/infer/decoder/attention.rs` does.
  Per-file calls are made at decomposition time.
- **Home of GPU-tier laws.** "Law in seiche, kernels in conatus":
  `seiche/laws/` holds each law and its CPU tier; `conatus::resident` holds
  the kernels and grid buffers; seiche gains an optional
  `conatus[resident]` dependency, minding conatus's existing dev-dependency
  on seiche. The alternatives were everything in `conatus::resident`, or the
  medium as a numen grid field.
- **The picker.** "Ordinary laws with a CPU tier": GPU-tier laws append to
  `PhysicsLaw` and each has a CPU tier, so every host shows them; GPU is a
  backend, not a separate list. The alternatives were a separate GPU catalog,
  or a `needs_device` filter.
- **Density in the browser.** "GPU on both, after P5": Density's GPU tier
  lands on turnstone and the web tree, reusing the repulsion work's
  non-blocking readback; the CPU tier serves hosts with no device. The
  alternative ran the web on the CPU tier first.
- **The repulsion lag.** Asked whether a one-step force lag is accepted, kept
  synchronous on native, or avoided by keeping the nodes resident, Mark
  asked: "What would rapier do if 3? Elaborate". Open until answered.
  *Answered and ruled (2026-10-01):* told that under a resident mode rapier
  stops advancing the nodes, so node–node hard contacts, per-node materials,
  node joints and two-way scene contact are lost for those laws and drag
  needs a GPU pin mask, while under the lag rapier keeps all of it and only
  repulsion arrives one step late, Mark said: "Rapier seems the fallback in
  any case, so 1 is necessary; i would like to see 2, or 3 if there's some
  benefit to defaulting to the gpu, in addition to 1". So the one-step lag
  with rapier keeping its role is built first and is the fallback
  everywhere; a resident mode follows on top of it, either above a node
  threshold (2) or as the default where a device exists (3), chosen on
  measured benefit. *Reading, not ruled:* the 2-or-3 choice returns to Mark
  with numbers.
- **A GPU-tier law lane.** Asked whether the catalog's 2D, rapier-only bound
  holds, Mark chose "Add a GPU-tier law lane": a follow-on for laws that only
  make sense at GPU scale, still 2D. The alternatives were keeping the bound,
  or opening 3D layouts. This amends the scope line above ("Not in scope:
  ... GPU tiers") for that lane only; it needs its own assessment.

## Progress

- 2026-09-02: assessed against the crates and the donor's archived docs;
  plan written; the first draft's collapse into tunings rejected by Mark
  and rewritten as laws × overlays × tunables.
- 2026-09-02: P1 landed — eight laws and six overlays in seiche, the
  catalogs, setters and attribute builders in the canvas, the three saved-
  scene fields in graphshell; seiche 73/73, canvas 188/188 (one
  pre-existing clock flake, see Findings), graphshell scene tests 6/6
  under `web,personal-sync,native`. The wasm check of the web host's
  restore/save edit is blocked on the missing genet worktree.
- 2026-09-03: the genet worktree recreated at the pin, the P1 wasm check
  green; P1b landed — the petgraph sources (two kind sources, a mass
  source, two depth sources, the Skeleton overlay, weighted Stress) and
  their saved-scene fields; seiche 74/74, canvas 192/192, graphshell
  scene tests 6/6, wasm check green.
- 2026-09-03: P2's web half landed — the panel's law / overlay / source /
  profile controls, the Free arrangement, the chrome hint, the signature
  snapshot fields, the `select` / `check` verbs, twelve receipts green
  after four driver runs that found and fixed five defects (the pause
  toggle, the anchors, Orbit under damping, Charge's calibration, Still's
  explosion); `Hold` joins the laws. seiche 76/76, canvas 193/193. Receipt:
  `Code/testing/mere/physics_p2_receipt.md`. Native half open on the
  picker decision.
- 2026-09-03 (P3): the web host draws the remote board from the score
  each frame with no canvas in the path (`web.rs` `remote_scene`), and the
  fixture's append replaces the whole mounted scene; the WebRTC path
  diffs the sceno scene. Every appended fixture card shares the source id
  `card:0`, so a board must key by instance, not source.
- 2026-09-03 (P3): the canvas's `apply_strategy_positions` both re-seeds
  every listed body and records the anchor slots — there is no way to move
  a slot without teleporting the body — and `reconcile_derived`,
  `settle_physics` and `SETTLE_TICKS` are crate-private, so a host cannot
  add an item that is not a graph node, nor ask for a settle. (Corrected
  2026-09-03: a graph node *is* added incrementally, through `visit` /
  `add_node_at` → `mint_node_at`: reconcile, seed, settle, with existing
  bodies left where the simulation put them. The earlier wording "no
  incremental add" overstated it.) The board is its own stack piece for
  those reasons; the canvas keeps its shape.
- 2026-09-03 (P3): the machine's disk filled to zero bytes mid-session
  (a burst from another build; it released to 711 GB free on its own).
  A plan write in flight was truncated to an empty file and restored from
  git; nothing else was lost. Two cargo git checkouts this thread's pin
  attempts fetched (genet `b78e2b9`, mere `8bb15d7`) sit unused in the
  cache, about a gigabyte each, re-fetchable.
- 2026-09-03: Mark ruled the native half: turnstone, pin bumped, an
  **Arrange pane** (arrangement + physics + overlays + sources + profile on
  the settings-row controls), and every physics choice as a palette row.
  Landed in turnstone: `Action::{SetPhysicsLaw, SetPhysicsOverlay,
  SetPhysicsKindSource, SetPhysicsMassSource, SetPhysicsDepthSource,
  ApplyPhysicsProfile}` with palette rows derived from the canvas catalogs
  (`Physics:` ×11, `Overlay on/off:` ×16, `Profile:` ×18, `Kinds:` ×5,
  `Mass:` ×2, `Depth:` ×3, after the Layout group); `arrange_pane.rs`
  (kind `turnstone.arrange`, renderer `Arrange`, palette "Open Arrange
  pane"), whose applied rows leave as `ArrangeIntent`s the shell lowers to
  the same actions; `ViewIntentV1` carries the law, overlays and three
  sources beside the arrangement, saved with the session and re-applied on
  open; the observe snapshot's `arrange_rows` feed `assert row`;
  `scenarios/physics_native.scn` is the receipt: `RESULT ok`, two
  captures, turnstone 30/30 in the targeted tests. Native receipt notes in
  `Code/testing/mere/physics_p2_receipt.md`. P2 is complete on both hosts;
  P3 next.
- 2026-09-03: P3 landed — `PhysicsBoard` in the canvas crate, the web
  host's remote board on it, `physics_remote_board.scn` green over the
  WebRTC fixture (Charge separates the appended card, Orbit keeps
  moving). P4 (drag and add) next.
- 2026-09-04: the physics **backend became a stack type**, on Mark's
  ruling. `mere-canvas`'s private `physics` module moved to
  `seiche::runtime` whole: `Physics` (inline / actor), `PhysicsCommand`,
  `PhysicsUpdate` and `TICK_DT` are seiche's public runtime now, with
  armillary an optional dep behind a default `actor` feature, so a
  consumer that wants only the integrator (a wasm build, a batch layout
  job) takes the inline backend and no threading runtime. The canvas
  consumes it unchanged. `PhysicsBoard` holds a `Physics` and a
  `LayoutView` instead of a bare `Simulation`, so a board ticks inline on
  wasm and can `offload` onto an actor thread on native; the settle budget
  it kept by hand is the backend's now. One new primitive closed the read
  gap the switch exposed: `Physics::refresh`, which folds the current
  layout into a view **without stepping**, so a host that syncs bodies and
  draws in the same frame sees them at their slots rather than one tick
  late. seiche 77/77 with the actor test moved across and green in both
  feature configurations; canvas 194/194 (the clock flake passed this
  run); `physics_remote_board` re-run against the WebRTC fixture,
  `RESULT ok`, 37 steps, 852 frames, three captures.
- 2026-09-04, the three optimizations Mark asked for after the extraction:
  (1) `LayoutSnapshot` carries the bodies' kinetic energy, the actor keeps
  the last folded figure, and `Physics::kinetic_energy` answers the same
  question on both backends — the receipts' energy floor works offloaded
  now. (2) The web semantics mirror recomputed `layout_stats` — a pairwise
  pass over every node — every frame; it now recomputes only while the
  canvas reports motion, on the frame after it stops (the final tick moves
  bodies and reports rest in the same call), and on any chrome change.
  (3) The standalone `graphshell-web` manifest optimizes the dynamics
  crates alone in dev builds (rapier2d, parry2d, nalgebra, simba, seiche
  at opt-level 3), so settle frames run at speed without slowing an
  incremental host build; the override reaches the wasm build only
  because that manifest is its own workspace. One rebuild of 7m44s, then
  Springs, Orbit, Stress, Still, Profiles and the remote board all
  `RESULT ok`; seiche 77/77 in both feature configurations, canvas
  194/194. Not done: wiring `PhysicsBoard::offload` — no native host shows
  the remote board yet, so there is nothing to call it.
- 2026-09-04 (P4 web): a composite `drag` verb does not work. The queued
  pointer events dispatch back to back in one JS turn and the canvas never
  moves the body, so the gesture is three verbs a receipt steps a frame
  apart. Found by a diagnostic scenario, which is also how the two
  thresholds below were measured rather than guessed.
- 2026-09-04 (P4 web): `data-node-count` reports the app host's authority
  graph, which a canvas-level add does not touch; the physics acts on the
  canvas's own graph, so the receipt reads the new `data-canvas-nodes`.
- 2026-09-04 (P4 web): the laws do not share one post-drag signature.
  Springs, Charge, Stress and Energy reclaim the node (`drag-return >= 40`
  after five seconds); the restless five keep energy; Anneal is a search
  that cools, so it owes displacement rather than a steady energy; Still
  holds the drop point within five pixels. A moving law also carries the
  node within the release frame itself, so the "released where it was
  dropped" check needs sixty pixels of tolerance there against twenty for
  a resting law.
- 2026-10-01: Anneal no longer moves pinned (kinematic) bodies, per Mark's
  "Seiche: skip non-dynamic bodies". Its `set_translation` had overwritten
  a dragged node's kinematic target, so a released node snapped back. No
  other law, overlay or Hold writes positions. The new seiche test fails
  without the fix and passes with it. The tree drag receipt now passes three
  runs in a row
  ([one-tree plan](2026-09-25_graphshell_one_tree_plan.md) §6).
- 2026-10-02: the dynamics grammar brief was written and its ten forks
  ruled (§3, P7). P7 moved to the
  [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) as its
  tracks G1 to G6 (F2), and P7's text here is kept as history.
- 2026-10-02 (P5a, branch `gpu-repulsion`): the tiled half of the kernel
  landed. `kernels::exclude` is `NodeExclusion`'s law (inverse square, hard
  floor, cutoff) over every pair through shared-memory tiles;
  `conatus::resident::Exclusion` uploads padded positions, launches it and
  returns a `PendingExclusion` whose `try_take` never blocks (`wait` is the
  native-only blocking form for tests). Receipts in `conatus/tests/exclusion.rs`:
  agreement at 1k and 10k (Findings), and the positive control, where 200
  unlinked bodies settle with 0 overlaps on the CPU and on the device (600 of
  600 ticks dispatched, spread 623.0 both) and 520-544 overlaps with the
  strength's sign flipped on the device side. The existing resident receipts
  stay green (4/4). Logs: `Code/testing/mere/gpu-repulsion/`. Not built: the
  cell list, the lagged seam (P5b) and the host wiring (P5c), which wait on the
  forks put to Mark the same day.
- 2026-10-02 (P5a, after the rulings): the cell list landed. Nexus's binning
  is ported by hand to CubeCL in `conatus::resident::binning` (Apache-2.0, a
  retained-license row in `LICENSES.md`): a count pass, the Blelloch exclusive
  scan with its auxiliary levels, the cursor copy and the atomic-slot scatter,
  over a dense grid of cells a hair wider than the cutoff whose bounds the host
  takes from the positions it uploads. `kernels::exclude_cells` (ours) walks
  the three-by-three block around each body. `Exclusion` takes the cells at or
  above `DEFAULT_CELL_THRESHOLD` (4,096, settable) unless the grid would
  exceed four cells a body, when it stays on every pair. Receipts: the scan
  is exact at 1, 255, 256, 257, 4,097 and 70,001 elements (three levels); both
  passes agree with the CPU law at 1k, 10k and 50k (cells worst 1.7e-5,
  5.4e-5, 9.0e-5; pairs 2.9e-6, 1.2e-5, 1.8e-5); the positive control holds on
  both (0 overlaps against 541-544 sign-flipped); a four-body layout a
  million units wide stays on every pair. Cost per call, pairs / cells / CPU
  in ms: 2,000 0.69 / 0.75 / 14.7; 4,096 0.87 / 0.79 / 43.6; 10,000 1.38 /
  0.93 / 159; 50,000 10.5 / 3.9 / 2,302; 100,000 57.7 / 7.5 / not run. So the
  4,096 default is this machine's measured crossover.
- 2026-10-02 (P5b): the lagged seam landed in seiche. `LaggedRepulsion`
  (submit / poll / in-flight count) sits beside the synchronous closure;
  `LaggedLane` holds the bookkeeping (body order and step per submission, the
  newest answer, the limit N, and `LaggedStats`: device steps, CPU steps,
  submissions, failures, mismatches); `ForceContext` carries one `repulsion`
  field and the step clock; `NodeExclusion` uses the lane and runs its CPU law
  whenever the lane returns nothing. `Simulation::set_lagged_repulsion` and
  `repulsion_stats`, `PhysicsCommand::SetLaggedRepulsion` and
  `Physics::set_lagged_repulsion` reach inline and offloaded simulations. The
  `gpu` feature (`conatus[resident]`) adds `seiche::gpu::PhysicsDevice` (one
  CubeCL client per host device, cloned; threshold, N and the cell threshold
  ride on it; shared counters readable across an actor) and
  `DeviceRepulsion`. Receipts: six seam tests on timed mock evaluators (an
  answer applies from the next step; N = 1 refuses a three-step-old answer and
  N = 3 takes it; a refused device stays on the CPU path; failed answers are
  counted and covered; a changed body set discards its answer; lagged forces
  reach the bodies and respect the threshold), and three on the device: a
  2,000-node settle at threshold 0 meets the CPU's bounds (0 overlaps, spread
  1511.1 against 1511.2, energy within 0.1%) with 591 of 600 steps on the
  device at 2.86 ms busy a tick against 14.5; the sign-flipped device law
  leaves 536 overlaps; a device lost after 100 submissions hands every later
  step to the CPU with the CPU's spread. seiche 96/96 (92 without `actor`).
- 2026-10-02 (P5c): the hosts. Pictograph feature `gpu` adds
  `Canvas::set_physics_device` and `PhysicsBoard::set_physics_device` (a
  device set before `offload_physics` rides the simulation onto the actor,
  one set after arrives by command; a law switch keeps the lane) and
  `physics_device_for(&WgpuHandles)`; mere gains `canvas-gpu` (pictograph
  `gpu`, opt-in, per the F5 ruling), graphshell gains `canvas-gpu`
  (`RemoteBoard::set_physics_device`), and graphshell-web turns both on. The
  web tree builds one device from `ProducerContext.core`'s handles on the
  producer's first frame and hands it to the canvas and the remote board;
  `gpu=off`, `gpu_threshold` and `gpu_max_stale_steps` are page options; the
  tree snapshot gains `physics-device` and the lane's counts; `log-physics`
  writes them into the receipt; past 512 nodes spread and overlaps come from
  `Canvas::layout_stats_without_stretch` (a grid, same definition).
  Receipts, bundle `89b75bb4`: the eleven law receipts plus profiles, add and
  drag green at the default threshold, and the eleven again at threshold 0
  with the device asserted on (the eight `NodeExclusion` laws served 99 to 447
  steps on the device; log `Code/testing/mere/gpu-repulsion/diagnostics/law-receipts-threshold-0-bundle-89b75bb4.log`). `p5_tree_cpu_settle_2000` (2,000 unlinked nodes) and
  `p5_tree_gpu_settle_2000` meet the same bounds (0 overlaps, spread 1,071
  against 1,074, energy 441k against 426k) with 413 of 418 steps on the device
  at N = 9 and the physics stage at 6.1 ms a frame against 81.9; at the ruled
  N = 3 the device served 9 steps and the receipt fails its count. pictograph
  3/3 device receipts. The web threshold default is left at 1,000 and N at 3
  pending Mark's call on the readback finding above; the native crossover
  (about 400, N = 1) is turnstone's to set when it is wired.
- 2026-10-02 (P5c, third round): Mark's rulings carried out. The
  `capture.rs` flag committed by pathspec (`af7d2b5f`; cambium-genet-web-host
  8/8). Main merged (`c4097a1e`): its genet repin (`b1eb3af` to `bd3e8861`)
  moved 19 genet packages in the web lock, one genet revision in the graph,
  the CubeCL and Burn family still pre.2 (lock SHA256 `a915fa23...`, copied
  to `Code/testing/mere/gpu-repulsion/web-Cargo.lock`). The web defaults are
  N = 9 and threshold 400 (`13910c40`), and `p5_tree_gpu_settle_2000` runs
  at them with no `gpu_*` options. Receipts on bundle `3a82eca7`: the eleven
  law receipts plus profiles, add and drag green at the defaults; the
  2,000-node settle green, 413 of 418 steps on the device, physics 4.9 ms a
  frame, spread 1,075, no overlaps, energy 425k; its CPU twin green, 89.3 ms,
  spread 1,071, energy 441k. A `git archive` export of `13910c40` builds for
  wasm offline both with a freshly generated web lock (4 m 06 s; the CubeCL
  family resolved to pre.2, see Findings) and with the recorded lock.
