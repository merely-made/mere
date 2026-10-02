# Physics Catalog Plan

**Date:** 2026-09-02
**Status:** in progress (P1 landed 2026-09-02; P1b, P2 on both hosts and P3 the remote board 2026-09-03; the runtime extraction 2026-09-04; P4 web half 2026-09-04, closing with the Graphshell tree port per the 2026-10-01 rulings in §5).
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
- 2026-10-02 (P6a): the canvas settle is a tick budget (`SETTLE_TICKS`,
  360), not a rest test, so a law that converges slower than six seconds is
  cut off mid-way unless it is a living law. Density advects by writing
  translations (FlowAdvect's pattern, and the only one a pin control can
  fail: rapier ignores `set_linvel` and `add_force` on a kinematic body), so
  its bodies carry almost no velocity and `physics-energy` does not read its
  motion.
- 2026-10-02 (P6a): re-splatting every tick, Density's rest is zero gradient
  *at the nodes*, not an even field: node j pushes node i with weight `m_j`
  through the diffusion kernel, so room grows with mass but compressed
  (logarithmically past one diffusion length), and a mixed-mass rest keeps a
  density CV near 0.25 where uniform mass reaches 0.05. On the 200-node
  generated graph the mass/area Spearman plateaus near 0.82 (degree mass)
  and 0.72 (PageRank) whatever the diffusion length (0.5, 1, 2 spacings),
  diffusivity (12k to 100k) or Jacobi count (24, 64).
- 2026-10-02 (P6a): at a fixed diffusion length the 128² grid's `α` in cells²
  scales as `(128·L/side)²`. On the 12-node sample (side 330, L 96) it is
  about 1 360, 24 Jacobi sweeps a tick leave the field about a second behind
  the nodes, and a diffusivity above 12k oscillates (overlaps, rank swinging
  to −0.6). Working at a coarser level (L ≤ 4 cells) cures the lag and cuts
  the 200-node step from about 1.6 to 0.3 ms, but drops the sample to an 8²
  grid where it reads 0.57.
- 2026-10-02 (P6a): Gastner–Newman's own evolving field (splat once, diffuse,
  never re-splat) was prototyped beside the re-splat law: for point masses at
  frame-rate steps it is unstable (78–1 153 overlaps on the generated graphs,
  rank ≤ 0.55). It stays in the code as `DensityFlow::Gastner` only as a
  probe option. *(Annotated 2026-10-02, second round: Mark chose this
  field; with CFL substeps it is stable, and it is now the only flow. The
  re-splat flow, the Sea boundary and the coarse working level were
  removed.)*
- 2026-10-02 (P6a, second round): stable for point masses once each tick is
  cut into substeps no longer than it takes the fastest node to cross half
  a cell, the field diffusing by the same substep; no tick in any probe hit
  the 64-substep cap. A truncated Jacobi solve drifts the field's total
  (0.6% over 400 steps on a 32² test), so the grid rescales to the total
  the walls conserve. The settled overlaps are 0 to 10 on the generated
  graphs, against Springs' 38 to 173 from the same seed.
- 2026-10-02 (P6a, second round): one pass of the evolving field is a map of
  the seed. It cannot correct what it did not start with: on the 200-node
  graph it settles by tick 60 at Spearman 0.60 (degree), whatever the
  seconds (2, 4, 6), initial blur (0.1 to 0.5 spacings), grid (64², 128²)
  or CFL fraction (0.25, 0.5); and from a clumped uniform seed, where
  rapier's contacts move bodies off the flow, it rests at density CV 0.34
  with 44 overlaps. Repeated passes, each splatting the nodes where the
  last left them, close both gaps: one-second passes reach 0.81 at 3 600
  ticks (64²), and the clump CV 0.05 to 0.07 with 0 to 1 overlaps.
- 2026-10-02 (P6a, second round): the cost is set by `D·dt/h²`, which the
  seconds-per-pass rule makes independent of graph size: about 190 Jacobi
  sweeps a tick at 128² and 4-second passes (10 to 17 ms native at n = 200),
  and four times that at 1-second passes (36 to 59 ms); 64² is 2.3 to
  4.3 ms at the same settings with the same ranks.
- 2026-10-02 (P6a, after the pure-Density ruling): the catalog lets any of
  the eight overlays join Density today, and they mix silently: each is a
  force rapier integrates as velocity in the same step Density writes
  translations. At the catalog's defaults over 900 ticks on the 200-node
  graph (Density alone: rank 0.60, 4 overlaps): Hub pull −0.33 with 510
  overlaps, Centre −0.21 / 410, Tide −0.26 / 409, Skeleton −0.21 / 362,
  Depth −0.04 / 230, Grid 0.32 / 80, Hub room 0.57 with density CV 7.3;
  Group pull changes nothing there (every generated node is its own site).
  Nothing refuses or marks the mix yet (`density_overlay_probe`).
- 2026-10-02 (P6a): no Nexus code is in the CPU tier: cloud-in-cell splat,
  Jacobi and a central-difference gradient are written from the textbook, so
  `LICENSES.md` gains nothing. Nexus's gather-style P2G (one thread per grid
  node over sorted particles) is the pattern for P6b's kernels.

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
- 2026-10-02 (P6a, first round, branch `density-cpu`): the CPU tier and the
  catalog entry landed; the defaults stop at forks. `seiche/laws/density.rs`
  holds `Density` (mass per node from the host, state behind a mutex),
  `DensityGrid` (cloud-in-cell splat, Jacobi sweeps of `(I − α∇²)u = ρ`
  warm-started from the last field, central-difference gradient sampled with
  the splat's weights) and the `DensityMedium` trait the GPU tier replaces;
  every contested choice is a field (`bounds` Walls or Sea, `flow` Resplat or
  Gastner, diffusion length, diffusivity, area per mass, the grid's working
  level). The advection skips non-dynamic bodies. Pictograph appends
  `PhysicsLaw::Density` (`density.gastner-newman`, "Density") to `ALL`, the
  law catalog and a bare `law.density` profile, reads the mass source
  (rebuilt on a mass-source change, graph-bound like Orbit), and
  `LayoutStats` gains `mass_area_rank` (Spearman of mass against discrete
  Voronoi area) and `density_cv` (CV of mass / area), shown on both
  Graphshell pages as `layout-mass-area-rank` and `layout-density-cv`.
  Tests: the Jacobi field matches the cosine-transform solution of the
  implicit step on a three-mass splat to 1e-4; a lone node feels no flow of
  its own; a pinned body never leaves its target, and with the skip removed
  the same test fails (pinned body at (9.6, 10.6) against (30, −20),
  `Code/testing/mere/density/seiche-density-pin-control.log`). On the
  12-node sample settled Density reads Spearman 0.83–0.88 and Springs 0.07;
  uniform mass from a clump reaches density CV 0.079 at 360 ticks and 0.050
  at 1 800 (n = 200; Springs 0.30); with the gradient term removed all three
  receipts fail (rank −0.395, −0.241, CV 0.310;
  `fault-no-gradient.log`), restored after. The 200-node receipt is ignored
  pending the forks: it reads 0.70 at 900 ticks, 0.77 at 1 800. Open, put to
  Mark: the field (re-splat or Gastner's evolving field), the boundary (walls
  or a sea), composition with edge springs, the settle (360 ticks is short of
  the plateau), the area measure, the defaults and grid level, and whether
  the 0.8 bar holds for PageRank mass. Not done this round: the wasm build,
  the headed receipts and the web step cost. Gates (offline, debug):
  seiche 96/96 default and 92/92 without default features; pictograph
  `--features canvas --lib` 263 passed, 3 ignored (the probes and the
  200-node receipt), the source-time clock flake passing on rerun;
  graphshell `--features web --lib` 228/228. Logs and probe tables in
  `Code/testing/mere/density/`.
- 2026-10-02 (P6a, second round, after the field, boundary, area and bar
  rulings; main `f02d9d35` merged into `density-cpu`): Density is
  Gastner–Newman's evolving field, walled. A flow splats the nodes once,
  smooths them by `initial_blur`, then each tick diffuses the field and
  carries the nodes in CFL-bounded substeps (`DensityFlowState` reports
  substeps, cap hits and the fastest speed); `passes` repeats the flow from
  where the last left the nodes. `density.rs` is 476 lines with its tests in
  `laws/density/tests.rs`. Tests: the Jacobi field against the
  cosine-transform solution (1e-4), the evolving field conserving mass and
  evening, no self flow, a crowd's heavy node with 1.4 times the median
  room and no cap hit, and the pinned body held (with the skip removed it
  ends at (7.8, 9.7) against (30, −20), `seiche-density-pin-control.log`).
  Receipts at named settings, since the defaults are open: the sample graph
  at eight 4-second passes (128², 900 ticks), and the 200-node graph at one-
  second passes (64², 3 600 ticks), degree mass, each ≥ 0.8 with Springs
  failing it; uniform mass from a clump to CV < 0.25 with no overlaps. With
  the gradient term removed all three fail (rank −0.395, −0.241, CV 0.310;
  `fault-no-gradient.log`). PageRank, recorded: 0.66 to 0.67 on the
  200-node graph at 30 one-second passes. Open, put to Mark: how drags,
  added nodes and mass-source changes re-enter the field (today a law
  rebuild, on a topology or mass-source change, starts a fresh flow; a drag
  is not re-entered); whether passes repeat, how many, and the settle that
  follows (F4); and the defaults: grid, seconds per pass, blur, sweeps
  (F6). Composition with edges waits on the composition assessment.
  Gates (offline, debug): seiche 96/96 and 92/92 without default
  features; pictograph `--features canvas --lib` 274 passed, 2 ignored
  (the probes); graphshell `--features web --lib` 230/230. Logs and probe
  tables in `Code/testing/mere/density/`.
