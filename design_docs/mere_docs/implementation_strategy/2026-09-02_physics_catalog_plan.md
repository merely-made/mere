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
| `orbit.gravity` | Orbit | n-body gravitation, mass by degree, tangential initial velocity, no rest; *2026-10-04:* only the orbital (tangential) motion frictionless, so radial motion settles (at no less than 0.82 whatever the host's damping), a weak centring well, and exclusion only to two node diameters | the graph as a solar system: leaves orbit hubs | degree |
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
- **Density's stop, reopened (2026-10-02, fourth round, `65ff1b11`).** The
  shift stop, recommended in the third round, ends the flow on a dip: the
  rank wanders pass to pass (sample 0.84 at pass 3, 0.745 at 15 to 30, 0.877
  at 60 to 90), so at that stop gen-50 read 0.786 against its 0.8 bar and the
  headed fixtures 0.71 (tree) and 0.53 (old page). Under the 120-pass cap
  every bar was met. Mark chose "Change the law": reduce the wander itself so
  any stop reads true. The alternatives were the cap alone, keeping the shift
  stop with a wider plateau bar, or a stricter shift. *Reading, not ruled:*
  the lane measures the wander's cause and brings formulations back as
  options with numbers; the shift stop stays the default meanwhile.
- **Density's stop, fifth and sixth rounds (2026-10-03, `density-cpu`
  `824ac8cb`, `76623e15`).** The wander's measured causes: rapier adds
  nothing after pass 1; on 200 nodes the 64² grid moves every node about 0.05
  spacings a pass without decay. Five formulations were prototyped, all off
  by default. Continuous renewal at 2 spacings first looked best (gen-50 at
  0.8 on "8 of 8" starts), but those eight starts were quarter-turn copies of
  two, and every earlier probe and receipt seeded the spiral in key order,
  which puts the highest-degree nodes at the centre. Over 16 inequivalent
  dealt starts the rank where the ruled stop lands is: base passes 0.722
  (gen-50) and 0.754 (200 nodes), renewal 0.736 and 0.768. The stop fires
  early: gen-50 stops at pass 6 reading 0.64 to 0.78, and the 120-pass cap
  reads 0.82 to 0.86 on all 4 dealt starts tried; on 200 nodes the stop reads
  0.73 to 0.80 and the cap 0.78 to 0.82. The 12-node sample's rest depends on
  its start (0.35 to 0.86 at the cap over 8 starts, mean 0.66, 2 of 8 at
  0.8). An axis-aligned seed meets cell-centred splat cells and shifts the
  stop's timing (200 nodes: 0° 0.737 at pass 18 against 0.79 to 0.81 at
  passes 32 to 53 for other turns), small beside the start's own effect.
  Asked what changes, Mark chose **"Later stop, measured"**: the passes stay,
  and a lane compares a stricter shift (0.01 to 0.02 for 3 passes) and a
  minimum pass count against the 120-pass cap over 16 dealt starts and brings
  the cheapest that meets the bars. The alternatives were always running to
  the cap, renewal anyway, or keeping the stop with lower bars. This amends
  the fourth round's "Change the law", whose premise (a wander the law must
  lose) the dealt starts did not bear out. Asked what the 0.8 bar means on
  small graphs, Mark chose **"0.8 from 50 nodes up"**: the rank bar applies
  from 50 nodes, and the sample and the headed fixture receipts check the
  flow qualitatively (rank rises above the seed's, CV falls), recording
  per-start values. The alternatives were a mean over a fixed set of dealt
  starts, or one pinned start per receipt. *Reading, not ruled:* receipts
  move from the key-order seed to dealt starts; the decay, quench,
  wall-inset, relax and renewal knobs leave the code once the stop lands,
  with their evidence kept in the logs.
- **Density's stop, seventh round (2026-10-03, `density-cpu` `73965233`,
  `e303e58b`).** Over the same 16 dealt starts (50 nodes / 200 nodes): the
  120-pass cap reads min 0.699 / 0.732, mean 0.796 / 0.780, 9 of 16 at 0.8 or
  above on 50 nodes; a minimum of 60 passes before the shift test reads min
  0.713 / 0.703, mean 0.790 / 0.777, 9 of 16, stopping at about 60 s; shifts
  of 0.02 and 0.01 still stop early on 50 nodes (6 and 5 of 16) and never by
  test on 200; the ruled stop reads mean 0.722 / 0.754. No variant puts every
  50-node start at 0.8. Density CV falls on every start under every variant,
  and Springs reads −0.44 to −0.55 on the same starts (the control). Asked
  which stop and bar, Mark chose **"Min 60, bar: all ≥ 0.7"**: the default
  becomes a minimum of 60 passes before the shift test, and the bar becomes
  every dealt start at 0.7 or above on both graphs, with the mean and the
  count at 0.8 or above recorded. The alternatives were the 120-pass cap with
  a mean bar near 0.8, or keeping every start at 0.8, which no stop meets.
  This amends the third round's 0.8 bar on gen-50 and the plateau bar of 0.7
  at 200 nodes into one bar over dealt starts.
- **Density's declaration (2026-10-03).** After G1, `Force` requires
  `Declared::terms()`; the lane wrote the grammar plan's sketch. Options: one
  K term with its density field as state in the Wasserstein metric weighted
  by node mass; K with position state, as Anneal and FlowAdvect; Em with an
  exposed energy ∫ρ ln ρ (amending the rule that a kinematic term is K); two
  terms, field diffusion and position advection. Mark chose **"Keep: K,
  field, Wasserstein"**. *Reading, not ruled:* G1's instruments skip K terms,
  so nothing measures Density's declaration; that is G5's concern.
- **Density's experimental knobs (2026-10-03).** Measured: quench and the wall
  inset had no effect, relax at 1.5 never stops and at 1.8 collapses, decay
  was worse everywhere, and renewal gained nothing from dealt starts but lets
  the field follow drags and contacts continuously. Options: keep renewal,
  drop the other four; drop all five; keep all five off by default. Mark
  chose **"Keep renewal, drop four"**: renewal stays as an option, and quench,
  wall inset, relax and decay leave the code, their evidence kept in the logs
  and here.
- **Density's suite cost, and seiche's speed (2026-10-03, `density-cpu`
  `0e1c745f`).** The dealt-start receipts take pictograph's default suite
  from 110 s to 401 s; one dealt start costs 105 to 130 s in a debug build,
  while all sixteen run in 133 s in release (three tests in parallel). Asked
  how to carry the cost (the bars in release with a quick default check; an
  opt-level override for seiche in dev builds; one start per graph; keep 401
  s), Mark answered: **"Changing the speed of seiche is a useful feature for
  dev and possibly otherwise. Let's consider how we might go from x0.20 to
  x50?"** *Follows:* none of the options as put; seiche's speed goes to its
  own design, put back to Mark. Density's merge waits on the interim choice.
  *Put back the same day:* asked whether he meant a simulation-speed dial
  (simulated seconds per wall second, 0.2x to 50x), seiche's compute speed in
  dev and test builds (a debug build runs about 14x slower than release; one
  Density start is ~115 s debug against ~8 s release), or both, Mark chose
  **"Both"**. Asked how the dial changes a run (ticks per frame at a fixed
  dt; a scaled dt; fixed dt up to a budget, then dt), he chose **"Ticks per
  frame, fixed dt"**: the trajectory is the same at every speed, fast-forward
  runs more ticks a frame up to a compute budget and reports when it cannot
  keep up, and slow motion steps less often and interpolates what is drawn
  between ticks. Asked what Density's merge carries meanwhile (release bars
  with a quick default; keep 401 s; hold the merge), he chose **"Release
  bars, quick default"**: all sixteen dealt starts are asserted in release as
  ignored receipts that Density-touching lanes and merges run, and the
  default suite keeps one quick sample check. *Reading, not ruled:* the dev
  build's speed is measured before any profile change, since an opt-level
  override touches every dev build of the crates it names.
  *Built and ruled (2026-10-04, `seiche-speed` `ba0e2f4f`):* the dial is
  `Physics::set_speed` (0.2 to 50, thousandths), with a per-frame step budget
  that reports the speed reached, and slow motion drawn between the last two
  ticks; 600 ticks are bit-identical at 0.2x, 1x, 3.7x and 50x for laws
  without NodeExclusion or Barnes-Hut, and a different dt differs (the
  control). The dev-build measurement corrected the figure above: debug is
  about 34x slower than release on one start (146.5 s against 4.3 s), and
  99.4% of a debug tick is Density's sweep. Mark ruled four questions:
  - the control, **"Speed select"**: presets 0.2, 0.5, 1, 2, 5, 10 and 50 in
    the physics section, default 1x, showing the speed reached when the
    budget binds (against a log slider, slower/faster buttons, or the page
    option only);
  - reproducibility, **"Sum in key order"**: NodeExclusion and Barnes-Hut sum
    bodies in key order, one sort a tick, so every law is bit-reproducible
    run to run and across speeds, their low bits shifting once (against a
    fixed-order map, or leaving it, where two 1x runs differed by up to about
    70,000 ULP after 600 ticks);
  - dev speed, **"Opt 3 in dev for physics"**: `[profile.dev.package]`
    opt-level 3 for seiche, rapier2d, parry2d, nalgebra, simba and glamx,
    taking one Density start from 146.5 s to 6.0 s and the pictograph suite
    from 102.5 s to 12.6 s, rebuilds unchanged and backtraces keeping
    file:line (against the test profile only, rewriting Density's sweep, or
    leaving it);
  - as built, ticked: **"8 ms budget default, Deadline pacing at 1x"** (the
    native actor sleeps out the rest of each tick's interval). "Remote board
    stays 1x" was left unticked and returns as its own question.
- **Energy's receipt passes off screen (2026-10-03).** In
  `p4_tree_physics_energy`'s settled capture one node and one edge are in
  view and the rest of the 11-node fixture has left it; the start frame
  already runs past the edges; G1's, P5's and Density's captures match, so it
  predates them. Options: a lane diagnoses whether the law's scale or the
  view's fit is the cause, fixes it, and adds a framing assertion to every
  law receipt with a positive control; the framing assertion only; a note.
  Mark chose **"Diagnose and gate"**.
  *Diagnosed (2026-10-03, `energy-frame` `39387df1`):* two causes. The law's
  scale: the fixture's two components settle about 9,600 world units apart
  under Energy's centring (√(r·ΣW/g) with r 60,000, g 0.02), converging slowly
  (3,882 apart at 6 s, 9,289 at 60 s), with edges at 524 against Springs' 171.
  The view: the page fits only at boot, on a non-Free arrangement and on Fit
  graph, never after a law switch, so every law is read at zoom 1 in a world
  window of about 982×627; even Fit graph cannot frame Energy (zoom 0.064
  against the floor of 0.1). The framing check (`Canvas::layout_framing`,
  `layout-outside == 0` on every law receipt, a planted off-screen node
  counted as exactly 1 for the control) failed before any fix: Energy 11 of
  11, Kinds 4–5, Anneal 2–3, Orbit 4, the profiles 1–5, edge cases in Charge,
  Stress and Flow. Asked how the view behaves, Mark chose **"Follow while
  playing"**: from any law, profile or Free switch the camera eases toward
  fit-to-content each frame while physics runs; any pan or zoom stops
  following, and Fit graph resumes it. The alternatives were fitting once at
  rest, fitting once a fixed time after each switch, or receipts pressing Fit
  graph with the product unchanged. Asked about Energy's scale (every
  candidate keeping the two-cliques claim, 3.35 to 4.66 against Springs'
  2.49 and a bar of 3.24), Mark chose **"Repulsion 6,000, centring 0.2"**: a
  retune under the same id, the islands 940 apart (ratio 5.1 against Springs'
  2.6), edges 159, converged by about 20 s, framed at zoom 0.45 by the
  following view. The alternatives were repulsion 6,000 alone (the same
  picture at 1/√10, still creeping at 60 s), a fit to the boot view
  (repulsion 10,000, centring 1.2, the islands mostly merged), or the
  coefficients kept with a lower minimum zoom. The framing check also found
  Orbit expanding without bound (extent 25,107 at 60 s, about 420 units a
  second, kinetic energy flat near 181,000: the catalog's exclusion repulsion
  outweighs gravity and counter-damping removes friction; without exclusion
  it reaches 1,026 at 60 s with 1 or 2 overlaps). Mark chose **"Own
  diagnose-and-retune lane"**: Orbit's framing check keeps failing until that
  lane lands. The alternatives were fixing it in the Energy lane, or
  accepting it as a living law and dropping its framing assert.
  *Orbit diagnosed (2026-10-04, `orbit-retune` `dc624c58`):* exclusion does
  76 to 83% of the terms' work in the first second, releasing the tight
  starting layout's stored energy (230,625 on P2 against the kick's 16,614);
  counter-damping returns exactly what damping removes, so the excess never
  leaves and the bodies coast outward (extent 404 at 1 s, 54,909 at 120 s).
  Centring alone bounds it but turns the motion into radial breathing
  (tangential share 0.04). Mark chose **"Frictionless orbits + centring"**:
  counter-damping cancels damping only on each body's tangential motion about
  the mass centre, so radial drift settles, a weak centring term (0.02) joins
  Orbit, and exclusion's reach shrinks from 1,000 to two node diameters
  (worst extent 2.42 times the first second's; tangential share at least
  0.90, coherence at least 0.92, at least 2.95 revolutions, no overlaps,
  hubs inside). The alternatives were the same with only the kick's rotation
  sense frictionless, or no centring with exclusion at three diameters.
  Following the view, the old page's drag receipt failed (a dropped node 21
  to 29 px from the drop against 20, the camera easing after release); Mark
  chose **"Drag stops following"**: a node drag stops following as a pan
  does, and Fit graph resumes it. Of three as-built recommendations he
  ticked **"Density gets the check here"** (the Energy lane merges main and
  asserts framing on Density's receipts); "Orbit's check unmarked" and "Face
  offset gets a lane" return as their own questions.
  The speed dial's remote board: Mark chose **"Follows the owner"**: a remote
  board runs at the speed its owner, the device running the simulation, has
  set, and shows it; the viewer cannot change it. *Reopened the same day:*
  the coordinator's question said the board's pace "is set where the
  simulation runs", which the speed lane found wrong: the owners (the C4
  host's `LiveEndpoint`, `mere_host`, djinn's residents) run no simulation
  for the board, and each viewer simulates its own `PhysicsBoard` from the
  owner's score, its law mirroring the viewer's canvas (P3). The question
  goes back to Mark with the finding. Put back with it (options: the
  viewer's own dial; the owner publishing a speed on the session's snapshots
  and diffs; 1x always), Mark chose **"The viewer's own dial"**: a remote
  board follows the viewer's Speed select, as its law follows the viewer's
  canvas, with no wire change.
  Asked about the GPU lane falling behind at 50x (117 of 638 steps on the
  CPU, against 5 at 1x), Mark answered: **"I mean, it's also true that I set
  those speed numbers arbitrarily. I feel like, some computers by virtue of
  having or not having a gpu or a stronger or weaker cpu will be able to
  simulate the scene faster. Is there a way to define a hardware-independent
  speed measure that allows people to use as much headroom as they want,
  between some reasonable hardware-independent resource bounds?"** Put back:
  speed already means simulated seconds per wall second; the 8 ms budget is
  what depends on the machine (half a 60 Hz frame, nearly all of a 120 Hz
  one). He chose **"Target + Max, budget as frame share"**: the presets stay
  and gain "Max", as fast as the budget allows, and the budget becomes a
  share of each frame (default 50%) instead of 8 ms, the page showing the
  speed reached (against a headroom dial alone, or keeping it as built). On
  the GPU lane he chose **"Staleness in ticks"**: forces are never more than
  9 simulated ticks old, so the CPU serves more steps at high speed and the
  reached speed shows the cost (against counting staleness in frames, or
  blocking on the readback at speed). Built literally, "a share of each
  frame" measured the page's frame interval, physics included, so on a page
  already slower than the display the budget grew with the frames it
  lengthened (300 nodes at 50x: frames near 500 ms, a budget near 250 ms).
  Asked which frame, Mark chose **"The display's frame"**: 50% of the
  display's frame period, the shortest recent interval approximating vsync,
  so there is no feedback and a slow page keeps its frame rate (against the
  measured frame as built, or a share of the frame's non-physics time).
  Built (`seiche-speed` `ad2420a4`, 50% of the shortest of the last 120
  intervals), the feedback was gone (300 nodes at 50x: the budget held at
  51.6 ms against 146 to 171 ms before) but the shortest interval is the
  page's own best frame, not vsync, when the page never keeps up: 24 to 30
  ms on light pages and 55 to 103 ms at 300 nodes on this machine, so the
  budget came out 12 to 52 ms. Mark chose **"Known rate, else capped"**:
  native hosts use the display's real refresh rate; on the web the period is
  the shortest recent interval but no longer than 1/60 s, so the budget is
  at most about 8.3 ms and less on faster displays (against half the best
  frame as built, or always capping at 1/60 s). The fast receipt's
  per-frame bound (budget plus the clock's 100 µs) saw 150 to 600 µs over in
  some windows, because the gate admits a tick on a forecast of its cost:
  Mark chose **"Gate keeps a forecast margin"**: the budget stops ticking
  when the time left is under the forecast tick plus a margin, so the
  overrun stays within the clock grain (against widening the bound by one
  tick's error, or leaving it).
  The face offset, diagnosed (2026-10-04, `tree-face-zoom`): the face is
  drawn right and the body wrong, on both pages. Pictograph's gnode style
  scales each body about its centre (the CSS default Livery follows since
  genet `62e1a0fa`, reaching mere at `8131d7a3`) while its arithmetic assumes
  the top-left, so below zoom 1 bodies drift down-right by (size/2)(1 − z),
  16.2 px at zoom 0.1; the old page has the same defect, so it cannot stay
  unchanged. Mark chose **"transform-origin in the gnode style"**:
  `transform-origin: 0 0` joins `GNODE_BOX` (`pictograph/src/canvas/build.rs`),
  both pages are corrected identically and zoom 1 is unchanged, proven by a
  body-geometry accessor and a unit test at five zooms (against the same
  declaration inline in `frame.rs`, or translating by the unscaled half).
  Weave (the workspace merge driver) mis-merged silently in four lanes on
  2026-10-03 and 04, each caught only by a compile failure or a comparison
  against `git merge-file`. Asked whether to keep it, Mark asked whether it
  was up to date and said he would rather unwire it than keep shooting his
  foot if it was. It was not (0.3.4 installed, built 2026-05-24; upstream
  0.5.4, 70 commits ahead, several fixes matching the failures). He chose
  **"Update, replay, decide"**: build 0.5.4, replay the four mis-merges
  against it, and keep weave only if all come out right, otherwise unwire it
  workspace-wide. *Replayed (2026-10-04, logs in `Code/testing/weave-054/`):*
  upstream `d73c4ae` (v0.5.4 plus 39 unreleased commits; every fix that
  matters landed after the v0.5.4 tag, so crates.io's 0.5.4 has none) got all
  four mis-merges right; across 201 replayed files it matched `git
  merge-file` wherever that was clean, conflicted wherever it conflicted,
  and refused 2 merges git would have made cleanly but broken, while 0.3.4
  reproduced every recorded loss and one more (`a31b9a14`, fields dropped in
  mesquite's `lane.rs`, restored at the time). Asked whether to install it,
  unwire weave, or wait for a release, Mark chose **"Install d73c4ae,
  quietly"**: the verified build is installed at a quiet moment, active
  sessions told first. *Installed 2026-10-04:* `weave 0.5.4` (d73c4ae) in
  `~/.cargo/bin`, the global driver string unchanged. `cargo install`
  rebuilt it, so its hashes differ from the tested build's, but replaying the
  four merges' 37 files with the installed driver gives byte-identical
  outputs and exit codes.
  The old page's drag receipt under the following view still failed (Stress
  22 against 20): following had zoomed out to 0.772 before the gesture, so
  the receipt's 220 px drag was about 285 world units. Mark chose **"Measure
  in world units"**: the gesture and the drag-return check work in world
  units, reading as today at zoom 1 (against resetting zoom first, or
  widening the thresholds). Orbit at host damping 0, where tangential-only
  counter-damping leaves radial motion unsettled (bounded within 6.3 times
  but breathing, tangential share as low as 0.08): Mark chose **"Radial
  floor at 0.82"**: Orbit settles radial motion with at least the old page's
  0.82 whatever the host's damping (at damping 0, worst 1.58 and 2.60 times,
  tangential share at least 0.89), against documenting it. And the muniment
  OPFS probe, whose page-error gate control needed the wasm-bindgen 0.2.126
  CLI: Mark answered **"move it and anything else to 0.2.129. let's stay
  with the newest."** Every wasm module in the tree moves to wasm-bindgen
  0.2.129, the OPFS probe and the two minimal repros included (burn plan,
  §13.40 or later).
  Orbit's API: asked how the tangential-only counter-damping meets seiche's
  public `Gravity::counter_damping: bool` (seiche 0.0.5 publishable), Mark
  chose **"Explicit enum, bump seiche"**: the bool becomes
  `CounterDamping::{Off, Full, Tangential}`, Orbit using Tangential, and
  seiche goes to 0.0.6, so callers must choose. Orbit's framing check under
  the following view: **"Leave it unmarked"**. The tree page's node faces
  drawn small and offset below zoom 1 (predating the follow change): **"Its
  own lane"**. The speed dial's fast receipt peaking at 8,100 µs against its
  8,000 µs bound, with Chrome's clock resolving 100 µs: **"Bound = budget +
  clock grain"**.
- **Receipts gate on page errors (2026-10-03).** The pre.4 lane found that
  wgpu 30.0.0 panics once per GPU-on page (`webgpu.rs:85`, "Unexpected
  error", then `RuntimeError: unreachable`) because wasm-bindgen 0.2.126 and
  later count only `undefined` as no error while the browser answers `null`;
  graphshell-web on main has it (0.2.127 with 30.0.0), and 117 receipt files
  carry it, P5's GPU receipts included, which passed because no headed
  receipt asserts zero page errors (the coordinator verified P5 without
  catching it). Asked whether receipts gate on page errors, Mark chose
  **"Gate every receipt"**: every headed receipt fails on any uncaught page
  error or panic, with a positive control, landing with the pin fix
  (wasm-bindgen 0.2.129 with wgpu 30.0.1, burn plan §13.38), and P5's GPU
  receipts are rerun under it. The alternatives were the GPU receipts only,
  or recording without failing.
- **P5's web defaults, undercut by pre.4 (2026-10-03).** On `burn-pre4-repin`
  the web page's frames take 557 ms against pre.2's 12.1 ms, GPU on or off,
  because the wasm module re-runs its static constructors (pliron's
  `inventory` registrations, which pre.4's `cubecl-core` pulls in) on every
  call into wasm. "Web N = 9, web threshold 400" was ruled on pre.2's frame
  times, so the crossover is re-measured once the constructors run once
  (burn migration plan, the pre.4 rulings of 2026-10-03). Main stays on pre.2
  until then; native is unaffected.

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
  *Annotation 2026-10-04:* cancelling all of the damping also kept all the
  energy exclusion released, and Orbit coasted apart (P5's rulings, under
  "Energy's receipt passes off screen"). `counter_damping` is now
  `CounterDamping::{Off, Full, Tangential}` (seiche 0.0.6, "Explicit enum,
  bump seiche"), and Orbit's is `Tangential`: only each body's tangential
  velocity about the gravitational mass centre, against the system's drift,
  is driven back, so radial motion and drift settle and the orbits do not;
  they settle at no less than `Gravity::RADIAL_FLOOR` (0.82, "Radial floor
  at 0.82") whatever the host's damping.
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
  closes it; open for Mark. *Ruled 2026-10-02:* asked first, Mark replied
  "Wait. Those should be bumped, no?"; told that pre.4 was published
  2026-09-22 and that branch `burn-pre4-repin` (23 commits ahead of main, last
  2026-09-30) holds the migration with ruling 411's allocator diagnosis open,
  he chose "Finish pre.4 now, no interim pin": a lane takes the pre.4
  migration to its gates and merge, and main is not exact-pinned meanwhile.
  The alternatives were an interim `=` pin while the migration finished, or
  bumping conatus alone.
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
- 2026-10-02 (P6a, third round): at the ruled defaults (64², one-second
  passes, blur 0.25) the 200-node graph's mass/area rank does not settle:
  over ninety passes it wanders between 0.73 and 0.79 (degree; PageRank 0.60
  to 0.70), and each pass still moves the nodes about a twentieth of a
  spacing on average. The 12-node sample (0.84 by pass 3, then 0.74 to 0.88)
  and gen-50 (0.78 to 0.87) wander too, more narrowly. A per-pass test on
  the layout therefore stops at different places by its kind: a field-CV
  test (the fresh splat's CV changing under 1–5%) stops every graph by pass
  4–9, the 200-node graph at 0.69–0.75; a shift test under 0.05 spacings for
  three passes stops it at pass 18 (0.76), under 0.02 never (the cap ends
  it). Trace: `Code/testing/mere/density/probe-convergence.log`.
- 2026-10-02 (P6a, third round): explicit Play sets the settle budget to
  `u32::MAX` (`canvas/input.rs:622`, "run so I can watch"), so after Play the
  canvas never rests under any law and the tree's `wait` cannot see a law
  stop; Density's own demand for ticks (`physics-continuous` on the tree)
  is what drops when its passes end. On the fixture graph it dropped near
  frame 3 700 after Apply.
- 2026-10-03 (P6a, the wander): rapier is not in it. Per pass the law now
  records the flow's own net displacement of each node beside the observed
  one, and after the first pass the remainder is 0.0000 spacings on every
  graph, with no overlaps or near contacts; zeroing velocities after each
  write leaves the traces identical, and holding nodes at the walls instead
  of a radius inside them changes the 200-node graph little (final 0.771,
  last two thirds 0.709 to 0.802). Two things move instead. On the 200-node
  graph at 64² every pass moves the nodes about 0.05 spacings (5 px) and
  never decays, the rim twice the interior and light nodes most; at 128² that
  falls to 0.01–0.02 and the plateau rises to 0.82, so the grid's resolution
  (a 22-px cell against a 25-px blur) is the large graph's cause. On the
  small graphs the motion does decay (the 12-node sample to 0.0006 spacings
  by pass 90) while the rank keeps moving: slow creep after an early stop,
  and on the sample, whose masses take three values, a 1–2% area change that
  flips pairs across tied mass groups. Over eight starts the sample settles
  into one of two equilibria (the stop reading 0.84 or 0.53–0.59, half and
  half) under every formulation tried. *(Corrected 2026-10-03: those eight
  starts were quarter-turn copies of two, the probe turning the seed by
  π/4 under a square's symmetry; see the sixth-round finding.)* Logs: `probe-wander-*.raw`,
  `probe-starts.raw` under `Code/testing/mere/density/`.
- 2026-10-03 (P6a, sixth round): every Density probe and receipt so far
  seeded the spiral in key order, which on these graphs puts the
  highest-degree nodes at its centre, and the round-five start probe turned
  that seed by π/4, two classes under the square's symmetry. From sixteen
  inequivalent starts (golden-angle turns, each with its own seeded deal of
  nodes onto the spiral), the rank where the ruled stop lands is, base
  passes: the sample 0.458 (0.00–0.877), gen-50 0.722 (0.618–0.856), the
  200-node graph 0.754 (0.684–0.844), with 2, 1 and 1 of 16 at or over 0.8;
  renewal 0.291 / 0.736 / 0.768; decay 0.453 / 0.725 / 0.726; 128² 0.457 /
  0.729 / 0.717. On dealt starts the stop fires early: gen-50 under the
  passes stops at pass 6 at 0.636–0.775 where the 120-pass cap reads
  0.820–0.863, and the 200-node graph 0.726–0.804 against 0.781–0.819; the
  sample's cap reads 0.661 (0.35–0.86, 2 of 8), so a small graph's rest
  itself depends on its start. The field's domain does not: it is a square
  sized by total mass about a fixed centre (`seiche/src/laws/density.rs`
  256–261, centre at 215). Turning the key-order seed through a quarter
  turn gives no smooth curve: on the 200-node graph the axis-aligned
  starts read 0.737 and 0.747 (stopping at pass 18) and every other turn
  0.786–0.807, and the gap tracks when the stop fires (128² 0.706 / 0.748,
  both at pass 7; the domain moved half a cell 0.755 / 0.772, the aligned
  start's stop moving from pass 18 to 34; blur 0.5 0.698 / 0.713). Logs:
  `probe-orientation.raw`, `probe-starts-16*.raw`, `probe-anisotropy.raw`,
  `probe-sample-cap.raw` under `Code/testing/mere/density/`.

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
  failing it; *(Corrected 2026-10-02, third round: the 200-node
  figure was one sample of a trace that wanders between 0.73 and 0.79 over
  ninety passes; at the ruled defaults that receipt does not hold. See the
  third-round entry.)* uniform mass from a clump to CV < 0.25 with no overlaps. With
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
- 2026-10-02 (P6a, third round, after the second-round rulings; main
  `64f3351a` merged): repeated passes, the overlay refusal and the ruled
  defaults landed; the stop test is a fork. Each pass re-splats where the
  nodes stand; a law rebuild (topology, mass source) starts a fresh flow; a
  held (kinematic) body re-arms a stopped flow and keeps passes running
  while held. `Density` gains `DensityStop` (`Shift`, `FieldCv`, `Cap`),
  `patience` and `max_passes`, the per-pass `pass_history`, and seiche's
  `Force` gains a defaulted `wants_tick` that `wants_continuous_tick` reads,
  so the law keeps the host ticking past its settle budget until its passes
  stop. The catalog builds Density at 64², one-second passes, blur 0.25, and,
  pending the test, the cap alone: 120 passes. `PhysicsLaw::overlay_refusal`
  and `OverlayRefusal`: `set_physics_overlays`, `set_physics_law` and
  `set_physics_choice` return `Result` (turnstone will see an unused-result
  warning at its next repin); adding overlays to Density is refused with a
  reason and nothing changed, switching to Density drops the live ones and
  names them, a whole choice applies law and sources and refuses the
  overlays; the tree greys the eight boxes with the reason beneath
  (`disabled-overlays`), the old page disables the fieldset and shows the
  note. Tests: the stop by test, by cap, and the drag re-arm (seiche); the
  refusal through every setter with Springs as the control; Density running
  past the settle budget while Springs rests (pictograph). Receipts at a
  named candidate stop (`Shift(0.05)`, three passes): the sample graph 0.837
  at its stop, Springs 0.116; uniform mass CV 0.063 with no overlaps; the
  200-node receipt stays ignored, 0.737 at that stop. The gradient fault
  fails all three (−0.395, −0.241, CV 0.310 with 13 overlaps); without the
  dynamic-body skip the pinned body ends at (12.0, 14.4) against (30, −20).
  Web, bundle built offline and locked (`wasm-build.log`): headed
  `p4_tree_physics_springs`, `_stress`, `_still` ok; `p4_tree_physics_density`
  ok on the fixture (Springs control ≤ 0.79, the overlays greyed with the
  reason, then Density past the cap: the law's tick demand gone, rank ≥ 0.8,
  CV ≤ 0.35, no overlaps; a trace read 0.86 by frame 180, 0.68–0.75 around
  frames 960–1 260, 0.93 at the stop); the old page's `physics_density` ok
  (7 300 frames). All six headed runs ok again on the final bundle (SHA256
  `1061b48e…`, `bundle.sha256`; `headed-final.log`). Step cost at n = 200
  on the web tree, from the profiler's physics stage, three steps every
  frame: p50 8.5 ms, p95 14.6, max 17.1 a frame (about 2.8 ms a step)
  against Springs' 2.1 ms on the same graph, in the final run; an earlier
  run on the pre-format bundle, the machine less loaded, read 7.3 / 8.9 /
  10.1 against 1.6. Frames run 100 to 155 ms, mostly DOM mutation. Captures inspected
  whole-frame: the fixture settles into a compact, evenly filled square with
  the two hubs holding the open middle, the effect modest at eleven nodes;
  at 200 nodes the walls' box overflows the unfitted camera (111–134 of 200
  visible). The runner, `Code/testing/mere/density/run-density-scenario.ps1`
  (port 8805, its own profile), launches Chrome with native occlusion and
  background throttling off: two runs had stalled with the window covered
  and the document hidden. The shared cargo download cache was emptied
  around 18:00 by something outside this lane; the root workspace's locked
  crates were fetched again online (`cargo-fetch.log`, 729 crates). Gates (offline, debug): seiche 97/97 and 93/93 without default features; pictograph `--features canvas --lib` 276 passed, 2 ignored (the 200-node receipt and the convergence probe); graphshell `--features web --lib` 230/230.
  Open, put to Mark: the stop test (field CV, shift, or the cap alone), and
  the 0.8 bar on the 200-node graph, which no stop meets at these defaults.
- 2026-10-02 (P6a, fourth round, after the third-round rulings; main
  `d3874ff3` merged): the stop test is the catalog default
  (`DENSITY_STOP` `Shift(0.05)`, `DENSITY_PATIENCE` 3, the 120-pass cap
  behind it), and the receipts read the rank only after the flow reports
  stopped (`physics_tick_demand`; on both pages the `physics-continuous`
  observation, new on the old page). Native, at the catalog default: the
  12-node sample stops at pass 5 at 0.837 (Springs 0.116); the 200-node
  graph stops at pass 18 at 0.737 and holds the plateau bar of 0.7, its
  plateau (0.73 to 0.79 over ninety passes) in the test's message; uniform
  mass from a clump stops at CV 0.063 with no overlaps. **Two results
  contradict the ruling and go back to Mark.** gen-50 stops at pass 6 at
  0.786, under the 0.8 the same ruling set for it (its own measurement said
  0.78); the receipt is written and ignored with that reason. And the
  headed receipts at the new default fail on rank after the flow reports
  stopped: the fixture graph reads 0.71 on the tree and 0.53 on the old page
  against 0.8 (under the cap alone the tree had read 0.93, and its trace
  had climbed to 0.86 by frame 180 before dipping). With the gradient term
  removed every receipt fails (sample −0.395, gen-50 −0.246, 200 nodes
  −0.241, uniform CV 0.310 with 13 overlaps); restored. Bundle `71d48984…`.
  Gates (offline, debug): seiche 97/97 and 93/93 without default features; pictograph `--features canvas --lib` 277 passed, 2 ignored (the gen-50 receipt and the convergence probe); graphshell `--features web --lib` 230/230 on a rerun, the first run losing `carrier::tests::p2panda_murm_grant_is_refused_before_projection_bytes` to a projection-accept timeout in code this lane does not touch (`gate-graphshell-web-flake.log`); it passed twice alone.
- 2026-10-03 (P6a, fifth round, after "Change the law"; main `ac75982d`
  merged, two concurrent-append conflicts in this plan and one in the tree
  lane resolved): the wander diagnosed (Findings) and five formulations
  prototyped as Density fields, all off by default, the shift stop and the
  passes still the catalog's: `decay` (each pass's diffusivity a fraction of
  the last), `quench` (velocities zeroed after each write), `wall_inset`,
  `relax` (over-relaxation at each pass's end) and `renew` (continuous
  renewal: no passes, the field pulled every tick toward the nodes' current
  splat, its steady smoothing set in spacings). The grid moved to
  `laws/density/grid.rs` (density.rs 497 lines). Results, rank where the
  shift stop lands over eight starts (12-node / gen-50 / 200-node, share at
  or over 0.8): base passes 0.691 / 0.797 / 0.769 (4, 4, 2 of 8); `renew`
  at two spacings 0.554 / 0.819 / 0.799 (0, 8, 4 of 8; the 200-node range
  0.780–0.819, its shift decaying to 0.0016 by pass 90 and the rim no longer
  leading); `decay` 0.9 0.708 / 0.784 / 0.739 (it freezes the transient);
  128² 0.689 / 0.782 / 0.739 at 16 times the step cost (64–82 ms a tick
  native against 3–4); `relax` 1.5 never stops and 1.8 collapses the rank;
  `quench` and `wall_inset` change nothing. `renew` below about one spacing
  freezes the nodes outright (its time constant falls under a tick, the
  field becomes the raw splat, and a lone splat has no gradient). The
  fixture, headed, on a diagnostic bundle with `renew` at two spacings (not
  committed): the old page 0.82 at the stop; the tree, whose ticks per frame
  vary run to run, ≥ 0.8, 0.40, 0.93 and 0.82 over four runs, against 0.71
  and 0.53 under the passes. The gradient fault fails base and `renew` alike
  (the seed's −0.395 / −0.246 / −0.241 held); Springs stays negative (−0.47
  and −0.62 on the generated graphs). The web lock was taken from the P5
  lane's worktree, whose manifest matches main's. Gates (offline, debug): seiche 103/103 and 99/99 without default features; pictograph `--features canvas --lib` 277 passed, 5 ignored (the gen-50 receipt and the probes); graphshell `--features web --lib` 230 passed, 1 ignored, single-threaded, after two parallel runs lost `session_notices::tests::the_endpoint_is_asked_even_while_no_request_is_in_flight` and `carrier::tests::p2panda_murm_grant_is_refused_before_projection_bytes` to timing under load, in code this lane does not touch (each passes alone; `gate-graphshell-web-flake*.log`).
  Open, put to Mark: which formulation, if any, replaces the passes; and
  what the bar means on a small, symmetric graph whose stop lands in one of
  two equilibria by its start. *(Corrected 2026-10-03: the eight starts
  were two classes, and every probe and receipt seeded the spiral in key
  order, a favourable start; the sixth-round entry restates both questions
  from inequivalent starts.)*
- 2026-10-03 (P6a, sixth round): the start probe re-run from inequivalent
  starts, an orientation sweep, one cheap check per anisotropy candidate,
  and the stop against the cap on dealt starts (Findings). No formulation
  changed and no default changed; the probes are ignored tests in
  `pictograph/src/canvas/tests/density_wander.rs`. Open, put to Mark again
  from these numbers: whether the stop, not the formulation, is the thing
  to change (the passes' cap meets 0.8 on gen-50 from every dealt start
  measured); and what the bar is on the 12-node sample, whose rest itself
  ranges 0.35 to 0.86 by start.
- 2026-10-03 (P6a, seventh round, after "Later stop, measured" and "0.8
  from 50 nodes up"): `Density.min_passes` (zero by default: the stop test
  may end the passes only from that pass on) and the stop variants measured
  over the sixteen dealt starts (`density_stop_variants`; logs
  `probe-stop-variants-*.raw`). Rank where each stop lands, min / mean /
  count at the bar / mean passes (one pass is a second of flow): gen-50 at
  0.8 — ruled shift 0.05 0.618 / 0.722 / 1 / 6; the 120-pass cap 0.699 /
  0.796 / 9 / 120; shift 0.02 0.638 / 0.756 / 6 / 10.5; shift 0.01 0.655 /
  0.770 / 5 / 20.7; at least 20 passes 0.639 / 0.769 / 5 / 20; 40 passes
  0.671 / 0.789 / 7 / 40; 60 passes 0.713 / 0.790 / 9 / 60. The 200-node
  graph at 0.7 — ruled 0.684 / 0.754 / 14 / 29.5; cap 0.732 / 0.780 / 16 /
  120; shift 0.02 and 0.01 never stop before the cap there; 20 passes 0.700 /
  0.766 / 15 / 30.2; 40 passes 0.703 / 0.773 / 16 / 44.2; 60 passes 0.703 /
  0.777 / 16 / 63.8. The sample: density CV falls on all sixteen starts under
  every variant; the rank rises over the seed's on all sixteen under the cap
  (least gain 0.128) and the 60-pass minimum (0.084), on 14 or 15 under the
  rest. Springs on the same dealt starts reads −0.44 to −0.55 on both
  generated graphs. No default changed: the cap itself puts 9 of 16 gen-50
  starts at 0.8, so which statistic the bar is goes back to Mark.
- 2026-10-03 (P6a, after G1 landed; main `39254463` merged): Density
  declares its terms as the dynamics grammar plan's Findings sketched it —
  one term, `Topology::Medium`, `Kernel::Diffusion`, class K, kinematic,
  `State::Field`, `Metric::Wasserstein` weighted by each node's mass,
  observed by `MassAreaRank` (*Reading, not ruled*; returned to Mark as a
  fork with the alternatives). `physics_terms` counts twelve laws and
  passes all four tests: the declaration test accepts Density's row; the
  class-agreement table reads no Density row, because it reads K terms by
  no instrument (the probe reads `user_force` at `dt = 0`, and Density
  writes positions), while the same probe reads every force-currency row
  (its only disagreements are Charge's Barnes–Hut θ 0.5 rung, as before).
  Gates (offline, locked, debug): seiche 109/109, 105/105 without default
  features, 109/109 with `gpu`; pictograph `--features canvas --lib` 282
  passed, 10 ignored; graphshell `--features web --lib` 231 passed, 1
  ignored, single-threaded. Logs `gate-g1-*.log`, `physics-terms.log`.
- 2026-10-03 (P6a, eighth round, after "Min 60, bar: all ≥ 0.7", "Keep:
  K, field, Wasserstein" and "Keep renewal, drop four"; main `df55ae2b`
  merged as `86bb5c84`): the catalog's Density runs at least 60 passes
  before the shift test (`DENSITY_MIN_PASSES`). `decay`, `quench`,
  `wall_inset` and `relax` are gone from `seiche::Density`, the walls hold a
  node a body radius in, `renew` stays an option, and the declaration cites
  its ruling. The receipts moved to the dealt starts (`seeded_dealt`, now in
  `tests/density.rs`, shared with the probes). A start costs 105 to 130 s in
  a debug build (60 passes are 3,600 ticks of the 64² flow, and Springs'
  900 control ticks come on top), so the default suite runs a fixed subset,
  dealt starts 0 and 1 of the sixteen. The sixteen run as ignored
  `_from_all_sixteen_starts` receipts: 133 s in release, three at a time,
  after a 212 s build (`receipts-r8-all-sixteen-release.log`). On gen-50 and
  200 nodes every start is asserted at 0.7 or above and Springs below zero,
  with the mean and the count at 0.8 recorded. On the sample the rank must
  rise above the seed's and the CV fall. Over the sixteen, gen-50 reads min
  0.713 (start 4), mean 0.790, 9 at 0.8; 200 nodes min 0.703 (start 2), mean
  0.777, 7 at 0.8; Springs −0.32 to −0.72 and −0.35 to −0.51. The sample's
  rank rises on all sixteen (least gain 0.084, start 3) and its CV falls on
  all sixteen, the same numbers the probe read. The default subset reads
  gen-50 0.823 and 0.844, 200 nodes 0.731 and 0.814, sample 0.702 and 0.877
  from seeds −0.427 and −0.004. Density's ranks agree to the digit between
  debug and release; Springs' do not quite (gen-50 start 0: −0.445 debug,
  −0.466 release). The pictograph suite takes 401 s (110 s before); the
  200-node receipt finishes last, and the uniform-mass test now runs its 60
  passes too. Both pages record the layout where a law was applied
  (`canvas_physics::LawStart`) and expose `law-start-mass-area-rank`,
  `law-start-density-cv`, `layout-rank-rose` and `layout-cv-fell`. A
  `log-layout <label>` verb writes rank, CV, overlaps, spread and the start
  into the receipt. Density's headed receipts and a new control scenario on
  each page (`p4_tree_physics_density_control`, `physics_density_control`)
  apply their law while the boot arrangement holds the canvas paused, then
  return to Free, so both start from the boot layout: rank 0.574, CV 0.336,
  3 overlaps. A first sequence applied Density after Free and the overlay
  checks, and its start (0.370) was not the control's (−0.058), because the
  live law ran in between (`headed-r8-summary-seq1.log`). From the shared
  start, Density reads rank 0.856, CV 0.155, no overlaps, identical on both
  pages, and Springs reads −0.029 with CV 0.559 / 0.560. *Reading, not
  ruled:* the headed seed is the boot layout; the control is its own
  scenario from that start, not a run before Density's; the control asserts
  rank ≤ 0 (a margin of 0.03 on this start), no rise and no evening; the
  default subset is the first two dealt starts; and Springs is not asserted
  on the sample, where it reads −0.40, 0.31, 0.004 and −0.22 on starts 0 to
  3. Fresh bundle `2c72f96b…` (`bundle-r8.sha256`): the Density pair and
  their controls ok on both pages, the twelve tree law receipts ok, and the
  200-node cost run's physics stage reads p50 8.7 ms, p95 15.7, max 22.5 a
  frame against Springs' 1.5 / 1.9 / 2.2, its frames about 117 ms, mostly
  DOM (`headed-r8-*.log`). Captures inspected whole-frame; Energy's settled
  frame shows the fixture flown off screen, as in G1's and P5's captures.
  Gates (offline, locked, debug): seiche 109/109, 105/105 without default
  features, 109/109 with `gpu`; pictograph `--features canvas --lib` 283
  passed, 12 ignored, `physics_terms` 4/4 at twelve laws; graphshell
  `--features web --lib` 232 passed, 1 ignored, single-threaded. Logs
  `gate-r8-*.log`.
- 2026-10-03 (P6a, ninth round, after "Release bars, quick default"; main
  `9dd866e4` merged as `b7b26048`, docs only, so no headed rerun): the
  default suite now runs one Density check,
  `density_on_the_sample_rises_and_evens_past_the_settle_budget`. It takes
  the sample from dealt start 0 past the settle budget by frames alone (390
  ticks) and asserts that the law still asks for ticks, that the rank has
  risen above the seed's (0.175 from −0.427) and that the CV has fallen
  (0.207 from 0.414). Springs from the same start rests at its budget (the
  control). It takes 16.7 s in a debug build. It replaces the eighth
  round's default subset and the 200-node past-budget test, whose claim it
  carries. The release receipts are ignored tests: the three
  `_from_all_sixteen_starts` and `uniform_mass_spreads_evenly`, which now
  runs its 60 passes. One line runs them all (also in the module doc of
  `pictograph/src/canvas/tests/density.rs`):

  `cargo test --release -p pictograph --features canvas --lib tests::density:: -- --ignored --nocapture`

  They pass in 193 s, four at a time, after a 91 s build
  (`receipts-r9-release.log`). Density's 48 per-start lines match the
  eighth round's to the digit: gen-50 min 0.713, mean 0.790, 9 of 16 at
  0.8; 200 nodes min 0.703, mean 0.777, 7 of 16; the sample's rank rises
  on all sixteen (least gain 0.084) and its CV falls on all sixteen;
  uniform mass stops at CV 0.055 with no overlaps. Springs does not repeat
  between runs of the same code: −0.32 to −0.72 on gen-50 and −0.39 to
  −0.50 on 200 nodes here, against −0.32 to −0.72 and −0.35 to −0.51 in
  the eighth round, all below zero. The pictograph suite takes 134 s (401 s
  in the eighth round, 110 s before Density's receipts); its slowest test
  is now `physics_terms`' instrument agreement. Gates (offline, locked,
  debug): seiche 109/109, 105/105 without default features, 109/109 with
  `gpu`; pictograph `--features canvas --lib` 279 passed, 13 ignored;
  graphshell `--features web --lib` 232 passed, 1 ignored, single-threaded.
  Logs `gate-r9-*.log`, `r9-quick-check.raw`.
- 2026-10-04 (Orbit's own lane, after "Own diagnose-and-retune lane";
  `orbit-retune` from `d60c2b86`, diagnostic `3b79e414`): the cause shown,
  the law unchanged, the fix back to Mark as a fork. The instrument is
  `seiche/src/laws/gravity/diag.rs` (ignored tests). It runs the catalog's
  Orbit split into its terms, exclusion (`NodeExclusion::default()`,
  220,000/d² out to 1,000), the kick, gravitation (G 9,000, mass 1 + degree,
  softening 24) and counter-damping, on the P2 fixture and G1's generated
  40-node graph from the boot Spiral's shape (19√i), and books each term's
  work on the kinetic energy and its mean outward push per window, with
  damping and contacts as the residual. Its positive control, exclusion alone
  at no damping, books 234,313 of work against an energy drop of 230,237.
  Orbit's set has no centring term.
  *The cause.* In the first second on P2, exclusion does +176,463 of work,
  gravitation −18,228 and the kick +16,614 (83%, 9% and 8% of the three;
  gen-40 76%, 13%, 11%), and exclusion pushes outward 172.8 against
  gravitation's 16.8 inward (gen-40 245.1 against 43.3). Inside its cutoff
  exclusion outweighs gravitation 6 to 12 times per pair on P2 (220,000
  against 9,000 × (1 + degree)), and it releases the tight seed's stored
  exclusion energy, 230,625 on P2, where the kick gives 16,614.
  Counter-damping's work equals the damping's loss in every window (547,296
  each in the first second), so nothing removes the excess: past 6 s every
  term's work is near zero, the kinetic energy is flat at 211,068 (gen-40
  1,593,253), and the extent runs 404 at 1 s, 27,405 at 60 s and 54,909 at
  120 s (gen-40 670, 43,804, 87,713). Controls, as the largest extent over
  120 s in multiples of the first second's, P2 / gen-40: without exclusion
  16.5 / 32.4, gen-40 ending with 80 touching pairs (clumped cores and an
  evaporating halo, so removing exclusion is not enough); without
  gravitation 136.8 / 133.8; without the kick 137.3 / 132.4; without
  counter-damping 4.5 / 4.4, with the energy at 8 / 54 by 120 s (the
  2026-09-03 death); with centring added (`Boundary` 0.08) 4.1 / 4.0, bounded
  but a radial breathing, tangential share 0.04 / 0.06, the energy swinging
  between 324 and 157,982.
  *The candidates*, test forces only, over five spiral seeds (spacing 16 to
  40) at both pages' dampings (the tree page's 2.5, the old page's 0.82) for
  120 s. *Reading, not ruled:* the bar is an extent within 3× the first
  second's; from 6 s a tangential share and an angular-momentum coherence of
  at least 0.8; at least one revolution; energy above the receipts' floor
  of 1; no overlaps; hubs inside (mass against radius below zero). Failing,
  on the boot seed: an escape brake (unbound bodies damped), 18.6 / 20.3,
  because each body stays bound to its own cluster while the clusters fly
  apart; a cap at the kick's energy, 49.6 / 52.8; exclusion cut to two node
  diameters alone, 98.2 / 79.1, and with softening 72, 100.6 / 87.0.
  Failing on the sweep: centring 0.02 with today's counter-damping, worst
  6.3, tangential 0.28; counter-damping on orbital (tangential) motion only,
  with exclusion at two diameters, worst 2.9 / 3.0 / 2.7 / 4.3 (P2 at 2.5,
  P2 at 0.82, gen-40 at 2.5, gen-40 at 0.82), with coherence 0.23 on the
  tight seed at 0.82, where counter-rotating bodies left by the blow-out
  cancel and reverse the rotation; the same at three diameters, worst 2.8,
  coherence 0.35 on one seed; the law's own radial damping at 2.5, worst
  3.0, energy 19 on one seed. Passing on every seed: (A) orbital-only
  counter-damping, exclusion at two diameters and centring 0.02, worst
  1.88 / 1.60 / 2.42 / 2.01, least tangential 0.90, coherence 0.92,
  revolutions 2.95, energy 179, mass against radius at most −0.18, no
  overlaps; (B) as A, with only motion in the kick's sense frictionless,
  worst 1.88 / 1.83 / 2.16 / 2.08, least 0.92, 0.93, 3.42, energy 262, at
  most −0.15. Nearly passing: (C) that prograde form with exclusion at three
  diameters and no centring, worst 1.98 / 1.77 / 1.93 / 2.83, but 0.86
  revolutions and energy 58 on one seed. Back to Mark as a fork, A
  recommended. Gates on `3b79e414` (offline, locked, debug): seiche 109/109,
  105/105 without default features, 109/109 with `gpu` (6 ignored, the
  diagnostics); pictograph `--features canvas --lib` 279 passed, 13 ignored,
  `physics_terms` green; graphshell `--features web --lib` 232 passed, 1
  ignored, single-threaded. Logs `Code/testing/mere/orbit/` (`diag-*.log`,
  `gate-diag-*.log`).
- 2026-10-04 (Orbit's retune, after "Frictionless orbits + centring" and
  "Explicit enum, bump seiche"; `orbit-retune`, main `c7125cd2` merged as
  `90919d24`, retune `0d7583d4`). Only this plan changed on both sides; weave's
  merge of it matches a plain `git merge-file` three-way merge. seiche:
  `Gravity::counter_damping` is `CounterDamping::{Off, Full, Tangential}`;
  Tangential cancels the host's damping only on each body's tangential
  velocity about the gravitational mass centre, against the system's
  mass-weighted drift. *Reading, not ruled:* `Gravity::new` takes the choice
  as an argument, with no default, so no caller's meaning changes silently.
  The only callers were seiche's own (the P1 never-rests test keeps `Full`,
  its old meaning, at no damping) and the catalog; nothing outside mere
  names it. seiche 0.0.6, with the workspace dependency and the lock's
  seiche row only. The catalog's Orbit is `NodeExclusion` to two node
  diameters (`ORBIT_EXCLUSION_REACH`, 72), `Gravity` under Tangential and
  `Boundary` 0.02 (`ORBIT_CENTRING`); `physics_terms` gains
  `orbit.gravity[2]` as E, gravitation staying H and counter-damping N.
  *Bounded, and still an orbit.* The bar is the one stated with the fork: an
  extent within 3× the first second's over 120 s; from 6 s a tangential share
  and an angular-momentum coherence of at least 0.8; at least one revolution;
  energy above 1; no overlaps; hubs inside. The law's own Tangential over
  the five-seed sweep (`diag_orbit_ruled`) gives a worst 1.87 / 1.59 / 2.26 /
  2.09 (P2 at 2.5, P2 at 0.82, gen-40 at 2.5, gen-40 at 0.82), least
  tangential 0.90, coherence 0.92, revolutions 3.29, energy 198, mass against
  radius at most −0.16, no overlaps, within 0.15 of the test force it was
  built from on every line. Through the canvas on the tree page's path
  (Play, Free, Orbit; `orbit_stays_bound_and_orbiting_on_the_p2_fixture`, 30
  s by default, 120 s as an ignored receipt) the extent is 205 at 1 s and at
  most 246 over 120 s (1.20×), the energy 2,904 to 3,941 after the first
  second, 5.55 revolutions, and no node off the canvas through its own
  camera at any second, with the boot camera that main still has. seiche's
  `tangential_counter_damping_keeps_the_orbits_bound` carries the claim with
  `Full` as its control, which fails the tangential bar.
  *The plan's Orbit claims, re-checked.* The laws table (annotated above):
  gravitation, mass by degree, the kick and no rest hold, and "leaves orbit
  hubs" reads as hubs inside on every seed. P1's floor after 600 ticks: the
  seiche test, green. P2's energy at 1 s and 6 s: `physics_orbit` and
  `p4_tree_physics_orbit` ok. P3's board keeps moving: the board's unit test,
  green; its headed receipt needs the C4 WebRTC fixture, not built here, and
  was not run. P4's drag and add rows: `physics_drag`, `physics_add` and
  their tree versions ok. The law.orbit profile: `p4_tree_physics_profiles`
  ok. The never-rests list and G1's classes: green. The 2026-09-03 finding is
  annotated above.
  *Headed receipts* on bundle `edbee10a…` (wasm-bindgen 0.2.129 CLI, the web
  lock seeded from `0090ad99` with the seiche row bumped, now `3cce8fc5`),
  port 8845, under the page-error gate: `p4_tree_physics_orbit`,
  `p4_tree_physics_drag`, `p4_tree_physics_add`, `p4_tree_physics_profiles`,
  `physics_orbit`, `physics_drag` and `physics_add` all `RESULT ok` with no
  gate failures. The positive controls on Orbit, a planted throw on the tree
  page and a planted panic on the old page, fail as they must. Captures were
  inspected whole-frame: all eleven nodes on the canvas, compact. Orbit's
  framing check lives on `energy-frame` and was not run here (nothing
  cherry-picked); no node left the canvas at any second without the
  following view.
  *At no host damping (asked):* Tangential has nothing to settle with, so
  the blow-out stays and only the centring holds it. Worst 6.28 / 5.29,
  least tangential 0.08 / 0.46, median 0.32 / 0.63, hubs no longer inside
  (mass against radius up to +0.28): bounded, but a radial breathing, not an
  orbit. With a floor under the radial settling at the old page's 0.82 it
  passes: worst 1.58 / 2.60, least tangential 0.89 / 0.96; at 0.7, worst 2.50
  / 2.67. Back to Mark as a fork. Gates (offline, locked, debug): seiche
  113/113, 109/109 without default features, 113/113 with `gpu` (8 ignored,
  the diagnostics); pictograph `--features canvas --lib` 287 passed, 13
  ignored, `physics_terms` green; graphshell `--features web --lib` 235
  passed, 2 ignored, single-threaded; clippy clean on the changed lines.
  Logs `Code/testing/mere/orbit/` (`diag-ruled-1.log`,
  `diag-no-damping-1.log`, `graphshell-orbit-p2-120s.log`, `receipts-r1.log`,
  `wasm-build-1.log`, `gate-ruled-*.log`).
- 2026-10-04 (Orbit's radial floor, after "Radial floor at 0.82";
  `orbit-retune`, main `fdb1f5df` merged as `ffa90384`, the floor
  `e9182cc8`). Only this plan changed on both sides, and weave's merge of it
  matches `git merge-file`. `Gravity` gains `radial_floor`, default
  `Gravity::RADIAL_FLOOR` (0.82): under Tangential, where the host's damping
  is below it, the law damps radial motion and drift up to it itself, and at
  or above it nothing changes. *Reading, not ruled:* the floor is a seiche
  field and default rather than a catalog constant, since Tangential is new
  in 0.0.6 and Orbit is its only user. The laws-table cell and the
  2026-09-03 annotation say so.
  *Against the bar*, the five-seed sweep (`diag_orbit_floor`), worst extent
  as a multiple of the first second's, P2 / gen-40:
  - host damping 0: 1.56 / 2.71, least tangential 0.90 / 0.96, coherence
    0.92 / 0.98, revolutions 2.91 / 3.79, energy 579 / 18,972;
  - 0.7: 1.52 / 2.52, least 0.90 / 0.96, 0.92 / 0.98, 2.92 / 4.23;
  - 0.82: 1.54 / 1.98, least 0.89 / 0.96, 0.91 / 0.98, 2.98 / 4.11;
  - 2.5: 1.89 / 2.31, least 0.99 / 1.00, 0.99 / 1.00, 3.39 / 3.88.
  Hubs are inside on every seed (mass against radius at most −0.17) and no
  run has an overlap. The control, the same law without the floor at
  damping 0, reads 6.28 / 5.29 and least tangential 0.07 / 0.39. Through the
  canvas on the tree page's path at damping 0
  (`orbit_stays_bound_and_orbiting_on_the_p2_fixture_at_no_damping`, ignored,
  120 s): 248 at 1 s, at most 377 (1.52×), 4.61 revolutions, no node off
  the canvas at any second; at the canvas's default damping, unchanged
  (1.20×, 5.57). seiche's `the_radial_floor_keeps_the_orbits_at_no_damping`
  carries the claim, with the floor at 0 as its control, which fails on
  tangential share. Both seiche Orbit tests now run the full 120 s the bar
  was stated over: an earlier 30 s draft of this one asked for one
  revolution in 30 s, a quarter of the bar's window, and failed at 0.78.
  *Headed receipts* on bundle `3ff58d2a…` (0.2.129 CLI, web lock `3cce8fc5`),
  under the page-error gate: the seven of the retune round `RESULT ok` with
  no gate failures, and the planted throw and panic on Orbit fail. Captures
  were inspected whole-frame. Gates (offline, locked, debug): seiche
  114/114, 110/110 without default features, 114/114 with `gpu` (9 ignored);
  pictograph `--features canvas --lib` 287 passed, 13 ignored,
  `physics_terms` green; graphshell `--features web --lib` 235 passed, 3
  ignored, single-threaded, on the second run. The first run failed on
  `session_notices::tests::the_endpoint_is_asked_even_while_no_request_is_in_flight`
  (polls 1 against more than 1), the timing flake the Density and G1 lanes
  logged; it passed alone and in the rerun. The default page-path test adds
  about 20 s to the graphshell suite in debug (30.8 s before this lane,
  51.9 s after the retune). Logs `Code/testing/mere/orbit/`
  (`diag-floor-1.log`, `graphshell-orbit-p2-floor.log`, `receipts-r2.log`,
  `wasm-build-2.log`, `gate-floor-*.log`, `gate-floor2-graphshell-web.log`).
- 2026-10-03 (Energy's off-screen receipt, branch `energy-frame` `629968cf`,
  per "Diagnose and gate"): diagnosed and gated; the fix is put back as forks.
  *Cause.* Two parts. The law's scale: on the P2 fixture (11 nodes, two
  components of 5 and 6, 10 relations) Energy's centring alone holds the
  components against the all-pairs repulsion, so they settle near
  √(r·ΣW/g) = √(60,000 · 31 / 0.02) ≈ 9,600 world units apart. It converges,
  slowly: 3,882 apart at 6 s, 7,232 at 20 s, 9,289 at 60 s (seiche replica,
  kinetic energy 368 at 60 s); edges settle at 524 against Springs' 171. The
  tree page read the same course from its own state: extent 1,692 × 1,771 by
  frame 30 (10 of 11 off), 5,701 × 9,174 at frame 1,800; the old page
  3,405 × 9,121. And the view: the page fits only at boot (to the Spiral,
  119 × 115), on a non-Free arrangement and on Fit graph, never after a law
  switch, so every law is read at zoom 1 against a 982 × 627 (tree) or
  1,282 × 722 (old page) world box. Even Fit graph cannot frame Energy: it
  needs zoom 0.064 and the canvas stops at 0.1. Springs on the same path stays
  inside, 457 × 470 at frame 1,800. *The gate.* `Canvas::layout_framing`
  counts node centres off the viewport through the canvas's own camera; both
  pages publish `layout-outside`, `layout-extent` and `view-extent`; the
  eleven law receipts and the profiles assert `layout-outside == 0` on the
  frame each second capture shows, and resting laws again after the final
  settle. The control (`*_framing_control`, both pages) plants a node 4,000 px
  off the canvas under Still: counted exactly, and an `== 0` assert fails on
  it with `got '1'`. *Reading, not ruled:* the margin is 0 (a centre inside
  the canvas rectangle; panels the old page overlays on its canvas are not
  subtracted), and those two moments are the stated ones for living laws too.
  *Before any fix* (bundle `10ac4316`, two rounds): Energy
  fails on both pages (11 of 11 on the tree; 8, then 11 on the old page),
  Kinds (4-5 tree, 1 old), Anneal (2-3 tree, 4-5 old), Orbit (4, tree), the
  tree profiles (1-5); Charge (one node, both rounds), Stress and Flow (one
  node, first round only) sit at the tree's edge and pass on the old page;
  Springs, Flock, Sync and Still pass everywhere. Orbit, met
  on the way, expands without bound: 25,107 units at 60 s, linear, energy flat,
  because exclusion outweighs gravity while counter-damping removes friction
  (1,026 at 60 s without exclusion). Gates on the instrument: seiche
  102/98/102, pictograph canvas 276, graphshell web 231 single-threaded. Logs
  and the sweep of candidate tunings: `Code/testing/mere/energy-frame/`.
- 2026-10-04 (the fix, branch `energy-frame`, per "Follow while playing" and
  "Repulsion 6,000, centring 0.2"). *Follow.* The canvas gains
  `set_view_follow` / `view_follows`: while physics plays, each frame eases
  the camera toward the camera `fit_to_content` would install (zoom
  geometrically, the world point at the viewport centre linearly, a 0.25 s
  time constant in host time, or one tick a frame where the host gives none).
  A wheel pan or zoom, a middle-drag, an orbit drag, `set_camera` and the two
  centring commands stop it; it holds while paused and while a node or the
  camera is being dragged. Graphshell's `canvas_physics` turns it on at a law,
  profile or Free switch and `CanvasCommand::Fit` turns it back on, so both
  pages follow; it is off by default, so other hosts are unchanged. Both pages
  publish `view-follow`. *Retune.* `LinLogForce`'s defaults are repulsion 6,000
  and centring 0.2 under the same id; on the fixture's topology the islands
  sit 5.07 apart for their size against Springs' 2.57 (asserted at Springs ×
  1.3, the two-cliques claim's factor; two cliques 4.49 against 2.49). The
  tree page now reads Energy converging: extent 860 × 1,113 and kinetic
  energy 0.0 by frame 1,800, against 5,701 × 9,174 and still moving before.
  *The control* pans first (following stops), plants the node 4,000 px off
  (counted as exactly 1; an `== 0` there fails with `got '1'` on both pages),
  then Fit graph resumes following and frames all 12. *Receipts* on bundle
  `677a03a2`: the eleven law receipts, the profiles and the control pass on
  both pages, every framing assert at 0. Before (bundle `10ac4316`): Energy
  11 of 11 (tree) and 8, then 11 (old page); Kinds 4-5 and 1; Anneal 2-3 and
  4-5; Orbit 4 (tree); the tree profiles 1-5; Charge, Stress and Flow one node
  at the tree's edge. Follow alone (bundle `b59af687`, the first coefficients)
  still left Energy 2 nodes out on the tree, past the 0.1 zoom floor. Orbit
  passes its capture-moment check on both pages under follow (its expansion
  outruns the floor only later), so its lane's expected failure has no frame
  to mark at the stated moment; put back. Also run green: the tree drag and
  add, the old page's add, `p4_tree_controls`, `p4_tree_live_profile`,
  `p4_tree_elapsed`, `p4_tree_profile`, the two saved-graph receipts, and the
  2,000-node settles (GPU 413 of 418 steps on the device, spread 1,075). The
  old page's `physics_drag` fails its released-where-dropped bound (21 and 29
  px against 20) because the camera eases after the release; put back as a
  fork. Found on the way, not fixed: the tree page draws node faces small and
  offset up-left of their bodies below zoom 1 (reproduced by toolbar zoom
  alone, with following off); following makes zoom below 1 common there.
  Gates: seiche 103/99/103 (default, no-default, gpu), pictograph canvas 277,
  graphshell web 232 single-threaded.
- 2026-10-04 (main 69ba33d0 merged into `energy-frame`, per "Drag stops
  following" and "Density gets the check here"). *Merge* `9724a6d5`: of the
  eight files both sides changed, five auto-merged exactly as a plain `git
  merge-file` does; in `canvas_physics.rs` weave's result silently dropped
  main's `SETTLED_ARRANGEMENT` re-export, the test module's `use` lines and two
  of main's tests, so the plain three-way merge replaced it (its seam's missing
  brace restored); `web_product.rs` and this plan keep both sides. Every line
  either side added survives but two rewritten doc lines. *Drag* `38bd8b0d`: the
  press that becomes a drag (past the click slop) stops following; a click
  does not; the follow test drags and clicks a node. *Density*: its two law
  receipts and two Springs controls assert framing on the settled capture.
  *Round* on bundle `a2837089` (wasm-bindgen 0.2.129 with wgpu 30.0.1, web
  lock `0090ad99`, the page-error gate on): the twelve law receipts, Density's
  controls, the profiles and the framing controls pass on both pages; tree
  drag and add, old-page add, `p4_tree_controls`, `p4_tree_live_profile`, the
  two saved-graph receipts, `p4_tree_profile`, `p4_tree_elapsed` and both
  2,000-node settles pass (GPU 413 of 418 steps on the device). The must-fail
  framing control fails with `got '1'` and a planted throw fails on the gate,
  each on both pages. The old page's `physics_drag` still fails, Stress 22
  against 20 on a fresh profile (and Anneal 79 against 60 on a profile holding
  an earlier saved session). The camera holds from the press through the
  release (621.63, 393.68 at zoom 0.772 throughout), so it is not easing after
  the drop: following zoomed the view out before the gesture, its 220 px is
  about 285 world units, and the law pulls the node back further in the
  measured frame (18 px against 11 one frame after release). With the follow
  step disabled the receipt passes twice. Put back to Mark with the receipt
  unchanged. Gates: seiche 113/109/113, pictograph canvas 289, graphshell web
  235 single-threaded.
- 2026-10-04 (drag receipts in world units, `energy-frame` `bd790bcc`, per
  "Measure in world units"). The canvas gains `focused_world_position`,
  `world_point_at` and `screen_point_of` (through the camera; a round-trip test
  at zoom 0.772); both pages gain a `move-by-world` verb beside `move-by`, keep
  the drop point in world units at `release-at`, and publish
  `drag-return-world` beside `drag-return`, the tree also
  `drag-return-step-world`. `physics_drag` and `p4_tree_physics_drag` make
  every gesture and check in world units, thresholds unchanged; at zoom 1 each
  reads as its px twin. *Reading, not ruled:* the ruling names the old page's
  receipt; the tree's mirror is converted too, so both pages measure the same
  thing. Its release log shows why: the 220-unit gesture is 112 px under zoom
  0.51 and 76 px under 0.34. *Round* on bundle `37523a31` (built from the
  head; web lock `0090ad99`, page-error gate on), fresh browser profile: both
  drag receipts pass (the old page's after a launch stall left no progress
  file and a rerun passed), and so do the twelve law receipts, Density's
  controls, the profiles, the framing controls and add on both pages,
  `p4_tree_controls`, `p4_tree_live_profile`, the saved-graph pair,
  `p4_tree_profile`, `p4_tree_elapsed` and both 2,000-node settles (GPU 413 of
  418 device steps). The must-fail framing control and a planted throw fail on
  both pages. Gates: seiche 113/109/113, pictograph canvas 290, graphshell web
  235 single-threaded.
- 2026-10-04 (main 9ce5889f merged into `energy-frame` `d2ea7526`: the
  face-zoom fix and Orbit's retune). Of the nine files both sides changed,
  five auto-merged exactly as a plain `git merge-file` does; `view.rs` weave
  auto-merged where the plain merge conflicts (both added functions after
  `focused_screen_position`) and placed main's elsewhere, so it was taken from
  the plain merge by hand; `canvas_controls.rs` keeps main's `SetZoom` (it
  places the camera through `set_camera`, so a set zoom stops following like
  any zoom) beside Fit resuming following; `canvas_physics.rs` and this plan
  keep both sides. One semantic conflict no text merge sees: `Gravity::new`
  takes a `CounterDamping` now, and the Energy sweep's Orbit runs record the
  first build, so they pass `Full` (27,405 at 60 s, as recorded). The web lock
  moves to `3cce8fc5`, the pins' `0090ad99` with seiche's row at 0.0.6, the
  same lock the Orbit lane built with. Orbit's receipts now assert framing
  after the final settle too (`6640bb64`), Orbit being bounded. *Round* on
  bundle `949fbc5e`, fresh browser profile: the twelve law receipts (Orbit at
  both its moments), Density's controls, the profiles, the framing controls,
  drag and add pass on both pages, main's two face-zoom receipts pass, and so
  do `p4_tree_controls`, `p4_tree_live_profile`, the saved-graph pair,
  `p4_tree_profile`, `p4_tree_elapsed` and both 2,000-node settles (GPU 413 of
  418 device steps); the must-fail framing control and a planted throw fail
  on both pages. Gates: seiche 115/111/115, pictograph canvas 292, graphshell
  web 238 single-threaded.
- 2026-10-03 (seiche's speed, branch `seiche-speed`, Part 1: the dial, as
  ruled, "Ticks per frame, fixed dt"). Where a frame's ticks were decided:
  `Physics::advance_frame` (one a call), `Physics::advance_elapsed` (elapsed
  time over `TICK_DURATION`, capped at 50 ms and three steps) and the actor's
  `run` (a tick, then `sleep(TICK_DT)`); the canvas's `frame_observed` picks
  the first two, the web tree passes the animation timestamp, turnstone calls
  `frame()` with physics offloaded, and the remote board `tick()`s. Settle
  budgets (`SETTLE_TICKS` 360, `SIZE_RESETTLE_TICKS` 90), `never_rests` (a
  `u32::MAX` budget), Anneal's cooling, the lagged lane's staleness
  (`now - answer.step`) and Density's passes (`seconds` of `dt`, 60 ticks a
  pass) already count ticks; only the ambient backdrop counts frames, and it
  is not seiche's. Built: seiche's `Speed` (thousandths, 0.2 to 50) owes wall
  time times the speed in integer units on all three drivers, so 0.2x is one
  tick every five frame-equivalents exactly; `StepBudget` bounds fast-forward
  on the host's clock and never cuts a call below what real time would run in
  it; `PaceStats` reports ticks, the effective speed over 32 frames and
  whether the budget bound; below real time the simulation leads by one tick
  and the snapshot is drawn between the last two (draw only). The canvas and
  the board carry it, and the web tree takes it as page options
  (`physics_speed`, `physics_budget_ms`, 8 ms provisional); the visible
  control is a fork. Receipts: one trajectory, bit for bit, at 0.2x, 1x, 3.7x
  and 50x on both drivers, the actor and the canvas (LinLog and Anneal; two 1x
  runs agree first; dt 1/30 and one tick more differ); settles of exactly 600
  (seiche) and 360 (canvas) ticks at every speed; slow motion drawn every
  frame and stepped about a fifth as often (web: 0.202x, 30 of 30 frames
  drawn, 7 stepped); the budget binding (virtual clock exact; canvas on the
  real clock within one tick's variation; web, 300 nodes at 50x, stepping at
  most 8.1 ms on the browser clock's 0.1 ms grain and reaching 1.6x against
  the page's 166 ms frames, with 24 nodes unbound at 39.6x as the control).
  The eleven law receipts plus profiles, add and drag are green at 1x on
  bundle `aa0041d9`, and `p5_tree_gpu_settle_2000` at 1x and at 50x (366 of
  389 steps on the device). seiche 111/111 (105/105 without `actor`),
  pictograph 279/279. Found on the way: `NodeExclusion` and
  `BarnesHutRepulsion` sum in `HashMap` order and are not reproducible run to
  run (24 of 24 bodies differ after 600 ticks, up to 33,559 and 69,959 ULP;
  every other law term, and `NodeExclusion` summed in key order, is). Nine of
  the eleven laws carry one of the two, so today a run is bit-reproducible,
  and the dial's identity provable, only under Anneal and Still. At 50x under
  the budget the 2,000-node page ran 244 ticks in 120 frames against 1x's 418
  until the budget's floor went in (the receipt fails without it). The actor
  now sleeps out the rest of its interval rather than a whole `TICK_DT` after
  each tick, so native 1x runs at real time instead of a tick's cost slower
  (not measured in turnstone). Logs: `Code/testing/mere/seiche-speed/`.
- 2026-10-03 (seiche's speed, Part 2: dev and test build speed, measured
  only). On a `git archive` export of `density-cpu` `dad99fde`, every variant
  passed as `--config` (no profile changed), one dealt start (`gen-50`, start
  0, 3,601 ticks) and pictograph's default suite, quiet machine, two samples:
  dev 146.5 s and 102.5 s; release 4.3 s and 8.7 s (34x on one start, not
  the 14x recorded above, which timed starts three to a core); seiche alone at
  opt-level 1, 2, 3: 97 s, 8.0 s, 7.7 s, the suite 81, 53, 63 s; seiche with
  rapier2d, parry2d, nalgebra, simba and glamx at 1, 2, 3: 30 s, 7.2 s, 6.0 s,
  the suite 24.7, 12.7, 12.6 s; `[profile.test]` at opt-level 1 for every
  crate 9.8 s and 9.7 s, after rebuilding every crate once (388 s); the test profile's registry crates at 2 with seiche
  at 0 147 s and 50.6 s, and with seiche at 2 7.3 s and 10.1 s. The rebuild
  after a one-line seiche edit stays 12 to 19 s at every dev and test level
  (release 38.6 s), and backtraces from inside `Simulation::tick` keep every
  frame with file and line at every dev and test level (release loses the
  inlined frames). Where debug's time goes: Density's force is 99.4% of a
  tick (rapier's step 0.6%); its Jacobi sweep runs 49 ns a cell in debug
  against 1.2 in release, of which overflow checks and debug assertions are
  about a fifth and the `Index` call chain with its bounds checks about two
  fifths (raw pointers: 28 ns); allocation is 14 a tick at every level. In
  release the bounds checks block vectorisation (indexed 1.2 ns, raw 0.24).
  The profile choice goes to Mark as a fork. Not verified: locals and
  stepping in a debugger (lldb lacks its Python DLL here); a sampling profile
  (the shell is not elevated, so the Windows profilers cannot run; the split
  above is by timers, opt-levels and a microbenchmark instead).
- 2026-10-04 (seiche's speed, the four rulings carried out, branch
  `seiche-speed`). Main `60eb5940` merged with its root lock kept exactly; weave
  had replaced the actor's `barrier`/`latest` impl with the second
  `impl ActorPhysics` and appended `DEFAULT_BOARD_ANCHOR_STIFFNESS` twice, both
  repaired by hand. "Opt 3 in dev for physics": the root manifest optimizes
  seiche, rapier2d, parry2d, nalgebra, simba and glamx in dev, and
  graphshell-web's manifest gains glamx so the two name one set; one Density
  dealt start then takes 6.1 s (146.5 s before) and the pictograph suite 55.5 s
  (117 s on the same merge before), the suite now bounded by
  `arrangement_roles::the_spiral_probe_through_the_canvas` at about 51 s, not
  by physics. "Sum in key order": `NodeExclusion` and `BarnesHutRepulsion` read
  `laws::node_positions`; every force set is bit-reproducible over five runs,
  and all twelve catalog laws, Density included, land on the same bits at
  0.2x, 1x and 50x after 150 ticks, one tick more differing under every law
  but Still; with the two files reverted both receipts fail. G1's
  class-agreement test passes with 44 of its 166 readings moved in their low
  digits and none beyond its tolerance. "Speed select": presets 0.2x to 50x in
  the physics section of both pages, 1x by default, applied when chosen
  (*reading, not ruled*), with "Running at 3.1x: the frame budget is full"
  beneath while the budget binds. Headed on bundle `211bd071` (wasm-bindgen
  0.2.129, web lock `0090ad99`), every receipt with zero gate entries: the
  eleven law receipts, profiles, add, drag and Density's two green; the slow,
  fast-control and both Speed select receipts green; two readings out of
  bound and left as they are: `p6_tree_speed_fast` stepped at most 8,100 us
  against its 8,000 us bound (the browser clock's grain is 100 us), and
  `p5_tree_gpu_settle_2000` at 50x ran 677 ticks in its 120 frames against
  1x's 418, so its frame-counted bounds miss (spread 1,135, energy 96k), with
  117 of 638 device steps stale. seiche 122/122 (116 without actor, 122 + 3
  with gpu), pictograph 292, graphshell `web` 234.
- 2026-10-04 (seiche's speed, the second round carried out, branch
  `seiche-speed`). "The viewer's own dial": `RemoteBoard::sync` takes the
  viewer's speed beside its choice, both pages pass the local canvas's, and
  the board's line reads "Board speed 2x, from your Speed setting"; a test
  runs 4, 20 and 4 ticks in four frames at the viewer's 1x, 5x and 1x. The
  owner-publishing half was dropped unbuilt beyond the viewer side. "Target +
  Max, budget as frame share": seiche gains `Speed::UNCAPPED` (Max: ticks until
  the budget is spent, 50x with no budget installed, which is its control);
  both Speed selects gain Max; the budget is 50% of the page's measured frame
  interval (its own frame timestamps, each new interval weighted 0.25, gaps
  over 1 s skipped) for the canvas and the remote board alike;
  `physics_budget_share` replaces `physics_budget_ms`; the note shows the speed
  reached whenever the layout moves (*reading, not ruled*: always, not only
  when bound). The fast receipt and its control are bounded per frame by that
  frame's budget plus the clock's 100 us grain. The fast receipt runs at Max,
  because on its 300-node page the budget feeds back: half of a frame that
  physics itself lengthens outgrows the catch-up cap at 50x (frames about 500
  ms, budget about 250 ms, 150 ticks a frame, about 5x), and at Max the page's
  frame reached 500 to 684 ms with stepping at 350 ms a frame; returned as a
  finding with three options (keep; a share of the display's period; a share
  of the non-physics time). "Staleness in ticks": no code change; the 50x
  2,000-node GPU settle is dropped from the batch, its bounds counting frames.
  Main `bd119a69` merged, the plan the only file both sides changed, and
  identical to `git merge-file`'s result; a retro-check of the previous merge
  (`48ead8a0`, whose second parent is `2d4b1ee9`, not the `60eb5940` its
  subject names; a git note says so) found every weave auto-merge identical to
  `git merge-file` after the hand repair. Headed on bundle `e68fbd0d`, every
  receipt green with zero gate entries: the eleven law receipts, profiles, add,
  drag, Density's two, slow, fast at Max (17x and 22x bound, worst over-budget
  23 and 91 us), its 50x control (34x unbound), both Speed select receipts with
  Max, and the 2,000-node GPU settle at 1x (413 of 418 device steps). seiche
  123/123 (117 without actor, 123 + 3 with gpu), pictograph 292 (its real-clock
  budget test failed once under 97% load, a frame preempted to 12.1 ms against
  3.7, and passed twice on a quiet machine), graphshell `web` 235.
- 2026-10-04 (seiche's speed, "The display's frame" carried out, branch
  `seiche-speed`). The step budget is 50% of the display's frame period, taken
  as the shortest of the last 120 intervals between the page's own frame
  timestamps (about 2 s at 60 Hz, half a second at 240 Hz), gaps over 1 s
  skipped and 60 Hz assumed until one is measured; the canvas and the remote
  board on both pages read it from one `web_speed::FrameBudget`, and both
  pages report the period beside the last interval (`display-period-ms`,
  `frame-interval-ms`). The before/after control is the same diagnostic
  scenario (`diag_speed_budget`, the 300-node tree page on CPU repulsion) on
  bundle `e68fbd0d` (half the smoothed frame) and `cf8ff645` (half the
  shortest interval). At 50x the budget grew from 146 to 171 ms over frames of
  291 to 342 ms before, and held at 51.6 ms over frames of 327 to 479 ms
  after; at Max it grew from 379 to 463 ms over frames of 758 to 927 ms
  before, and held at 27.4 ms over frames of 194 to 321 ms after. The feedback
  is gone, but the budget is not near half the display's 16.7 ms: no page here
  keeps up with the display, so the shortest recent interval is the page's own
  best frame (24.2 to 30.2 ms on the fixture and the 24-node page, 54.7 to 103
  ms at 300 nodes), and the intervals are not whole multiples of 16.67 ms, so
  no vsync quantum can be read off them. The budget comes out at 12 to 15 ms
  on light pages and 27 to 52 ms at 300 nodes, against about 8.3 ms. This went
  back to the coordinator as an open finding with three options (keep it as
  built; cap the period at 1/60 s; a known display rate where the host exposes
  one). The receipts keep their per-frame bound, that frame's budget plus the
  clock's 100 us grain. On bundle `cf8ff645` the fast receipt met it at
  exactly 100 us over, and the diagnostic runs saw 150 to 250 us in some
  windows, one tick run longer than forecast, so the bound can flake. The note
  showing the speed reached whenever the layout moves, not only when the
  budget binds, stays a *reading, not ruled*, kept as one on the coordinator's
  word. Main `8f61b367` merged under weave `d73c4ae` (`b6cc15fc`): the six
  auto-merged files are identical to `git merge-file`'s result, and the plan's
  one conflict, resolved theirs then ours with weave's `refused_by` line and
  markers removed, is identical to the same resolution of `git merge-file`'s.
  Main gave `Gravity::new` a counter-damping argument, so the reproducibility
  receipt passes Orbit's `Tangential` (`ddbfffd0`), and the web lock moves to
  `3cce8fc5` (seiche 0.0.6; the lock is not committed). Headed on bundle
  `71959113` (wasm-bindgen 0.2.129), 21 of 22 receipts green with zero gate
  entries: the eleven law receipts, profiles, add, drag, Density's two, slow,
  fast at Max (bound at 2.3x and 3.4x, the budget held at 42.45 ms, half an
  84.9 ms period, worst over-budget 50 us), both Speed select receipts with
  Max, and the 2,000-node GPU settle at 1x (413 of 418 device steps). The 50x
  control missed `effective >= 25` at 20.4x, unbound and 13.5 ms under its
  budget, at 76 to 83% CPU with another lane's rustc: at 50x the 50 ms
  catch-up clip caps the speed at 50 x 50 / frame ms, and its shortest
  interval was 48.5 ms against 30.2 ms the round before. Rerun with no other
  lane's rustc (52 to 58% CPU), the control passed twice (34.6x and 29.7x) and
  the fast receipt passed, but one of its windows ran 600 us over its budget,
  beyond the 100 us bound, which the receipt's final assertion does not see
  (its last window read 100 us): the flake named above, met. Gates: seiche
  125/125 (119 without actor, 125 + 3 with gpu), pictograph 294, graphshell
  `web` 238, mere and graphshell checked clean.
