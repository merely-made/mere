# Physics catalog

**Status:** 2026-10-28: cut from `2026-09-02_physics_catalog_plan.md`, archived in `archive_docs/2026-10-28/`. Built: the eleven laws and the overlays in seiche with the catalogs in the canvas, the petgraph sources, the levers and a receipt per law on both hosts (P1, P1b, P2), the remote board (P3), the public `seiche::runtime::Physics` backend, P4's web half, P5a to P5c (the GPU repulsion kernel, cell list, lagged seam, setters and the web tree), P6a (Density's CPU tier), the Orbit and Energy retunes with the view following while playing, and seiche's speed dial with its period estimator and worker tiebreaker. Open: P4's close, Turnstone's wiring of P5, P5d, the closing of P5, P6b (Density's GPU tier), P6c (its receipts), the GPU-tier law lane, and a short list of leftovers.

**For:** a catalog of distinct physics layout laws, meaning dynamical systems over the graph's bodies that produce different layouts because they are different physics, as a lever beside the arrangement catalog, plus the composable extra forces the donor's presets were made of, in the canvas, both Graphshell hosts and the remote board, each with receipts; and the GPU tier: repulsion on the host's own device with rapier as the fallback, and Density as the first GPU-tier law. Mark's 2026-09-02 framing opened it (there are catalogs of arrangements, presentations and scenes, so there should be one of physics layouts); his 2026-10-01 and 2026-10-02 rulings added P5 and P6.
Not here: P7, composition of several layouts at once (currencies, Meaning as a source, grouped laws, weighted lists, sequenced blends), which F2 moved to the dynamics grammar plan on 2026-10-02 and which is carried there as G1 to G6, together with the Bonds overlay on Density (`2026-10-28_dynamics_grammar_plan.md`); the physics scenes, tangibility and tactile-profile tails, which that plan holds because the archived tails say this plan is not their successor (same plan); Phase 4 of the Graphshell one-tree cutover, whose physics-panel scenarios close P4, and viewer budgets (`2026-10-28_graphshell_host_plan.md`); the Nexus adoption watch (`2026-10-28_conatus_engine_plan.md`); the Burn migration's remaining pins, wasm-bindgen and wgpu among them (`2026-10-28_distillery_models_plan.md`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings. The old plan's per-round ruling prose stays in the archived copy; the rulings that bind an open phase are carried below.

## Constraints

- **Laws are distinct dynamics, never tunings of one.** New law ids are for novel dynamics only; a scale version of an existing law stays under its id as a backend tier; tunings are welcome as profiles, arrangements and scenes over the laws, never presented as laws. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1539-1540, 1607-1614`.
- **Scope.** seiche stays rapier and 2D; the GPU-tier law lane is the one amendment. New integrators, the ambient backdrop sims, the donor's WASM layout mods and 3D layouts are out; matching, Steiner trees and cliques as groups are not taken. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:10-13, 1566-1568, 1687-1692`.
- **Rapier is the fallback everywhere.** The one-step lag with rapier keeping its role is built first; a resident mode follows on top, either above a node threshold or by default where a device exists, chosen on measured benefit. Under a resident mode rapier stops advancing the nodes, so node-node contacts, per-node materials, node joints and two-way scene contact are lost for those laws and drag needs a GPU pin mask. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:361-365, 1672-1686`.
- **GPU repulsion recipe.** `NodeExclusion`'s exact law (inverse square, `min_distance` floor, `cutoff`) as a CubeCL kernel on the host's own device, never a device of its own: tiled all-pairs below a node threshold and a cell list above it, the count, scan and scatter binning ported by hand from Nexus now; an error or missing adapter falls back to the CPU scan. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:339-360, 382-385, 1631-1638`.
- **The lagged seam.** A `LaggedRepulsion` trait beside the closure (submit and poll) in seiche's `gpu` feature, `ForceContext` taking one `repulsion` field and `Force` unchanged; the newest completed result applies for up to N steps and the CPU scan only past N (native 1; web 9 with a web threshold of 400, ruled on pre.2 frame times); readback never blocks. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:366-381, 498-506`.
- **Device and gating.** `Canvas` and `PhysicsBoard` take a cheap `PhysicsDevice` clone the host builds once from its handles; the `mere` feature `canvas-gpu` chains pictograph `gpu`, seiche `gpu` and `conatus[resident]`, opt-in per consumer; the threshold is a configurable node count per host whose default is that host's measured lagged-mode crossover, 0 meaning always GPU. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:386-399`.
- **Home and picker.** A GPU-tier law and its CPU tier live in `seiche/laws/`, the kernels and grid buffers in `conatus::resident` (seiche's optional `conatus[resident]` dependency); such laws append to `PhysicsLaw` as ordinary laws with a CPU tier, GPU being a backend and not a list. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1658-1667`.
- **Licensing.** Nexus-derived code is treated as Apache-2.0: a close port is a retained-license entry in `LICENSES.md` with the upstream notices; code merely informed by it stays MPL-2.0 with a credit line. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1649-1657`.
- **Density's law and tiers.** `density.gastner-newman`, label Density: nodes advect along the negative gradient of a diffused density grid until area follows mass (degree or PageRank). The field is Gastner-Newman's evolving field made stable by adaptive substeps; the boundary is walls; area share is raster Voronoi; passes re-splat where the nodes are; the CPU tier defaults to a 64 by 64 grid with 1-second passes and blur 0.25, the GPU tier to 512 by 512; the stop is a minimum of 60 passes before the shift test (shift under 0.05 spacings for 3 passes) with a cap of 120; renewal stays as an option; the declaration is one K term with the density field as state in the Wasserstein metric. The integrator is resident CubeCL with a kinematic mask and a grid-gradient force, GPU on both hosts after P5, reusing P5b's readback, the CPU tier serving hosts with no device. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:456-497, 513-523, 551-598, 1639-1643, 1668-1671`.
- **Density's bars.** The rank bar applies from 50 nodes up and asserts every dealt start at 0.7 or above on both generated graphs (gen-200's per-start floor is 0.68, dynamics F182), the mean and the count at 0.8 or above recorded; the sample and the headed fixtures check the flow qualitatively; the mass bar is Spearman 0.8 on degree mass with PageRank's figure recorded; Springs is the negative control, and its tree-page control asserts `layout-mass-area-rank <= 0.3` (dynamics F83). All sixteen dealt starts run as release receipts that Density-touching lanes and merges run, with one quick sample in the default suite. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:471-475, 558-581, 616-622`; `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:223, 431`.
- **Overlays on Density.** Refused with a reason unless admitted; Hub room, Centre and Tide converted are admitted (dynamics F73, F81), and Bonds only if rank holds at least 0.8. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:438-442, 494-497`; `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:199, 215`.
- **Receipts gate on page errors.** Every headed receipt fails on any uncaught page error or panic, with a positive control, landing with the wasm-bindgen 0.2.129 and wgpu 30.0.1 pin fix. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:959-971`.
- **The speed dial.** Simulated seconds per wall second at ticks per frame and a fixed dt, so the trajectory is the same at every speed; presets 0.2 to 50 plus Max, the budget a share (50 percent) of the display's frame; a remote board follows the viewer's own dial; the lagged-force staleness is counted in simulated ticks; a dial row that passes under load counts and a miss is rerun calm. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:598-650, 711-740, 862-870`.
- **Orbit's framing check stays unmarked** (ruled 2026-10-04, "Leave it unmarked"). From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:953-955`.
- **P4 closes only after the Graphshell one-tree plan's physics-panel scenarios land** (2026-10-01). From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1573-1577`.
- **Gates** rerun only when their inputs changed (dynamics F138, F139); a receipt or golden that moves stops the lane and is reported (F166). From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:339, 341, 397`.

## Phases

### P4. Drag and add, the close
The web pointer path reaches `pointer_down/up`, drag pins and re-settles under each law, and adding a node fans it out and settles; `physics_drag` and `physics_add` are green over all eleven laws on the web. The native half and the close wait on the tree port.
- Done when: the rows are green on the web and the native host shows the same by hand; the close-out is the Graphshell one-tree plan's physics-panel scenarios. The native "by hand" condition is unrecorded.
- Needs: `2026-10-28_graphshell_host_plan.md` (one-tree Phase 4)
- Rulings: P4 closes after the tree port (2026-10-01)
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:4, 320-338, 1570-1577`.

### P5c-T. Turnstone's wiring of the repulsion solver, and its repin to the dial
Turnstone is the native host and builds a `Canvas` per graph runtime. The solver and a `PhysicsDevice` from its own handles are to be wired through the setters, the native crossover (about 400 nodes, N = 1) being Turnstone's to set. Turnstone also pins a mere from before the speed dial: it repins, takes its display refresh rate from winit and gains a speed control.
- Done when: part of P5's list: a 2,000-node settle at threshold 0 passes the same energy, overlap and spread bounds as the CPU path in Turnstone, with the call count proving the GPU ran. For the dial, the source states the order only: Turnstone repins mere and takes its refresh rate from winit as the next step.
- Needs: P5a to P5c (merged); the dial's merge (merged)
- Rulings: lagged seam, device and gating, threshold (above)
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:4, 340-342, 357, 770-776, 1895-1896`.

### P5d. Resident mode
Measure the lagged mode against a resident mode (integration on the GPU, rapier's contacts, joints and materials dropped) at 2,000, 10,000 and 50,000 nodes on both hosts.
- Done when: the numbers are in front of Mark, for his choice between "resident above a threshold" and "resident by default where a device exists".
- Needs: P5a to P5c; P5c-T for the native host
- Rulings: the repulsion lag (2026-10-01)
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:361-365, 1672-1686`.

### P5x. Closing P5: defaults and receipts
The plan text marks none of P5's done-conditions met. The web defaults (N = 9, threshold 400) were ruled on pre.2 frame times and are re-measured once the pre.4 static-constructor cost is fixed (a Burn migration item). P5's GPU receipts rerun under the page-error gate.
- Done when: GPU and CPU forces agree to 1e-3 relative at 1k and 10k nodes; a sign-flipped kernel fails the overlap check (positive control); a forced adapter failure falls back and still passes; the eleven law receipts stay green at the default threshold; a 2,000-node settle at threshold 0 passes the same energy, overlap and spread bounds as the CPU path on the web tree and in Turnstone, the call count proving the GPU ran; physics cost per frame is recorded beside the CPU figures; P5d's numbers are in front of Mark.
- Needs: P5c-T; P5d; the pre.4 constructor fix (`2026-10-28_distillery_models_plan.md`)
- Rulings: receipts gate on page errors; web N and threshold
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:959-987`.

### P6b. Density's GPU tier
Grid splat, diffusion and gradient kernels in `conatus::resident` (fixed-point atomics or a gather for the splat, the particle-to-grid transfer ported by hand from Nexus's MPM), a kinematic mask on `integrate` (`kernels.rs`) so drag pins hold, nodes resident under this law with P5b's non-blocking readback, 512 by 512 by default.
- Done when: the GPU grid matches the CPU reference field to 1e-3 mean relative error; a dragged node holds under the GPU tier; n = 1,000 at 512 by 512 steps within 16 ms resident on the reference machine.
- Needs: P5a to P5c; the Nexus build blocker (NX below)
- Rulings: Density's integrator; GPU on both hosts after P5; home of GPU-tier laws
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:989-1010, 1639-1643, 1668-1671`.

### P6c. Density receipts
`physics_density` on the old page and the tree, native in Turnstone, and the GPU tier on both hosts after P5.
- Done when: settled area share correlates with mass at Spearman 0.8 or above (a new `LayoutStats` field) and uniform mass gives a low coefficient of variation of density; Springs fails that correlation (negative control) and removing the gradient term fails it too (fault injection); the CPU tier passes on the web; the eleven existing receipts stay green. The bars as since ruled are in Constraints (every dealt start at 0.7 or above, gen-200's floor 0.68, 64 by 64 CPU default).
- Needs: P6b; P5c-T
- Rulings: Density's bars; dynamics F83, F182
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:999-1010, 551-581`; `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:223, 431`.

### GL. The GPU-tier law lane
A follow-on for laws that only make sense at GPU scale, still 2D. Scent (chemotaxis), Current (SPH-carried) and Sheet (an elastic membrane) were the alternatives to Density. A mobile host for GPU repulsion waits for a host to exist.
- Done when: not written; the lane "needs its own assessment".
- Needs: P6b
- Rulings: add a GPU-tier law lane (2026-10-01)
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1615-1618, 1636, 1687-1692`.

### SR. Seiche actor left ticking by a dropped canvas
An offloaded simulation under a law that never rests holds a `u32::MAX` settle budget, and the actor exits on a closed channel only when the budget reaches zero, so dropping such a canvas leaves its actor thread ticking (`seiche/src/runtime.rs`, `run`).
- Done when: not written; the defect text names the cause and says "not fixed here".
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1375-1381`.

### NX. The Nexus build blocker and the licence entry
Mark ruled "fix now" for the Windows failure in the Nexus checkout, where `cargo-gpu-install` removes `Cargo.lock` when spirv-std comes from crates.io. The Nexus-derived binning port needs its retained-license entry in `LICENSES.md`. Neither has a completion line.
- Done when: not written; the ruling is "fix now", and a close port carries an entry with the upstream notices.
- Needs: -
- Rulings: the Nexus build blocker; licensing
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:382-385, 1644-1657`.

### FZ. Node faces below zoom 1
Pictograph's gnode style scales each body about its centre while its arithmetic assumes the top-left, so below zoom 1 bodies drift down and right by half the size times (1 minus the zoom); both pages have it. Ruled: `transform-origin: 0 0` joins `GNODE_BOX` (`pictograph/src/canvas/build.rs`). Landing is not recorded, and the tree page's small, offset faces below zoom 1 went to "its own lane".
- Done when: both pages are corrected identically and zoom 1 is unchanged, proven by a body-geometry accessor and a unit test at five zooms.
- Needs: -
- Rulings: transform-origin in the gnode style (2026-10-04)
- From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:899-909, 954-956`.

## Open questions

- The kind-source defaults taken in P1 (by site, by cluster, by degree; default site) were put "for Mark to confirm or overturn", and a relation-family kind source is "a small P2 item if wanted". From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1554-1561`.
- `PhysicsBoard::offload` has no native caller until a host shows the remote board. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1547-1552, 1792-1793`.
- The numerical cap for the plain viewer payload (a six-asset payload at 2,147,689 bytes brotli) "remains Mark's decision"; the scenograph editor plan records Mark raising the Graphshell viewer cap to 3.50 MiB per route artifact on 2026-10-16. Whether that ruling covers the plain payload, and which plan holds the budget. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:3157-3160`; `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1304`.
- Whether the speed-dial rows were ever confirmed calm: they passed only under load (no calm window arrived), and the 50x control's bar (half of the lesser of 50 times the page's 1x and the speed the budget fits) was ruled but its calm confirmation is not recorded. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:3090-3125`.
- Whether the page-error gate on every headed receipt and the move of every wasm module to wasm-bindgen 0.2.129 (the OPFS probe and the two minimal repros included) landed; both rulings carry no completion line. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:944-948, 959-971`.

## Findings

- 2026-10-02: Chrome answers a GPU readback about two frames after submit, so the newest answer was six steps old at every size from 128 to 2,000 nodes; that is why the web runs N = 9 and a threshold of 400. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:500-506`.
- 2026-10-03: under Burn pre.4 the wasm module re-runs its static constructors on every call into wasm, so web frames cost far more than under pre.2, GPU on or off; native is unaffected. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:972-979`.
- 2026-10-01: Nexus's rigid-body solver applies gravity only and has no repulsion force; it takes a host device through `WebGpu::from_device` and its step reads back without blocking; linking it adds about 164 crates and a second rapier and parry. Its MPM transfer, radix sort and cell binning are the patterns to port. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1624-1630`.
- 2026-09-03: `BarnesHutRepulsion`'s default strength was a third of `NodeExclusion` at a node diameter; Charge is built at 6,000 in the canvas, matched at contact. From: `archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md:1240-1243`.

## Progress

- 2026-10-28: cut from `2026-09-02_physics_catalog_plan.md`; nothing new landed.
