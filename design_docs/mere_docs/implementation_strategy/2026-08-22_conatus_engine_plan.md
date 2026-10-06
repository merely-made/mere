# Conatus Shared Spatial Runtime

**Date:** 2026-08-22  
**Status (2026-10-06):** active; body/runtime foundation and private-backend
integrity (`339e8567`) implemented. The first profile-local resident
body-position publication and the first product renderer tenant were
implemented in Isometry's `isometry-runtime`, since retired (relayed, §3), and
stand as historical receipts. Nexus admission probe blocked at its upstream
Windows shader build; scope corrected 2026-08-23. `quint` was folded into its
owners at `eae87153` (Existing pieces). 2026-08-26: Mesocosm's runtime
became the first product tactile consumer (terrarium picking over
`BodyWorld`, Rapier private), quint's `ResidentChunk` join (now
`conatus::resident`) was proven with per-brick patches, tracer-validated read
epochs, and allocator-observed bytes (V1b), and `conatus-brick` (`modulus`
since `33f9b6b6`) — the shared sparse-brick ABI both game vessels pin —
advanced on `codex/conatus-brick-lift` to `bd8f0044`.
2026-09-26: `modulus`'s shrinking-retarget defect fixed and its atlas sized
to the card by `AtlasLimits`, ruled by Mark (brick-atlas pass below).
2026-09-27: `modulus`'s `brick_dda` takes each voxel crossing afresh from the
eye instead of accumulating it and clamps its first voxel into the pointer
volume, and the same walk is public on the CPU as `BrickMap::trace`, all
ruled by Mark (brick-traversal precision pass below); on main since
2026-09-27 (`a404cd48`, `5e46956a`, `c3e054d6`).
2026-09-28: body-binding shape and the T2 voxel-store lane carried from the
wing's existing rulings into §1 and §2. Both are planned, not implemented;
body bindings remain document-only under ruling 346, and T2 waits for the
accepted pre.4 migration under ruling 363 (main is on pre.4 since
2026-10-05; see T2). This documentation pass neither implements nor
certifies the separate query-refresh API.
**Scope:** Build the shared spatial runtime. Mesocosm, Paredros, Isometry,
and Mere projections consume it through product-owned runtime profiles
instead of incubating spatial machinery in product-local probes.

## Scope correction (2026-08-23)

Ruled by Mark after the 2026-08-23 engine-stack review chain (stack review,
verdict, adjudication). The settled answer:

> Product profiles conduct. Conatus advances spatial state. Reusable
> mechanics can move early under settled ownership. Cross-product contracts
> remain provisional until a second game proves them.

The corrections, each carried into the body text below:

- Conatus is the shared spatial runtime, not the application engine. "The
  engine" is the composition: a product-owned runtime profile conducting
  clocks, triggers, input mapping, authorization, source bindings, and
  subsystem selection. Product clocks are not interchangeable
  (`mesocosm/design_docs/2026-08-18_engine_ecology_rulings_and_review.md`
  §2.8), and the implemented runtime already has the right shape:
  host-driven `step(steps)` advances exact steps, and frame changes publish
  on zero-step host frames.
- Seiche remains the graph-oriented 2D specialist, not an eventual adapter.
- The host/profile orders passes; allocation ownership follows advanced
  state (the host-conducts ruling,
  [spatial compute plan](../technical_architecture/2026-08-13_spatial_compute_plan.md)).
  Netrender's tenancy seam stays the device seam.
- The engine-owned render view becomes a lean spatial frame with no cameras,
  lights, sprites, or presentation policy (§4).
- Schedule phases are renamed around spatial work (§1).
- Scripts and peers submit product intents; raw remote `BodyCommand`s leave
  the architecture (§1).
- Source bindings are profile-owned; Conatus identities stay runtime-only
  (§1).
- The closing extraction rule splits into early mechanics and
  second-consumer contracts, reconciling this plan with the wing's
  two-consumer law.

## Direction

Conatus is the shared spatial runtime, not a collection of physics
experiments — and not the application engine. The engine is the composition
a product's runtime profile conducts: the profile owns clocks, triggers,
input mapping, authorization, source bindings, and subsystem selection, and
decides when to request a Conatus step, a Seiche relaxation, a field
evaluation, or an inference job.

Conatus owns reusable spatial machinery: the fixed-step spatial clock, body
identities, transforms, collision, field effects on the state it advances,
resident allocations for that state, generic procedural geometry,
spatial-frame preparation, and spatial system execution. A game owns its
rules, durable world meaning, assets, procedural content choices, and
presentation choices; its profile owns orchestration. Netrender owns device
tenancy and frame composition. Renderling is a 3D render tenant. Rapier is the
current CPU collision and dynamics implementation. Nexus and its Khal kernels
are source material for the scale at which the CPU backend stops being the
right implementation.

Ordinary unit, property, and integration tests remain part of runtime work.
Windowed demos, screenshots, receipt applications, and benchmark gates are
not the work queue. Performance work starts from a named engine workload and
changes the engine path itself.

## Existing pieces

| Piece | Role in the stack |
|---|---|
| `numen` | Serializable field and coupling vocabulary |
| `quint` | Field evaluation, CubeCL kernels, resident tensor/chunk allocations |
| `seiche` | Graph-oriented 2D dynamics specialist; shares contracts with the runtime where real, never an adapter over the 3D body API |
| `conatus` | Shared 3D body world and host-neutral runtime |
| `conatus-brick` | Product-neutral sparse brick ABI and ray-in WGSL DDA; never camera, material, or composition policy |
| Netrender | One-device tenancy and final frame composition |
| Renderling | 3D scene/render implementation, consumed as a tenant |
| Mesocosm voxel types | First source material for generic voxel storage, revision, dirty-region, collision, and meshing features; since 2026-08-26 Mesocosm is also a runtime consumer, holding `BodyWorld` tactile advice through its own `mesocosm-runtime` adapter |

*Annotation, 2026-10-06 (S14 pass):* `quint` is no longer a crate.
`eae87153` (authored 2026-08-31, on main 2026-09-02) folded it into its
owners: the resident allocations and chunks into `conatus::resident`
(`crates/conatus/conatus/src/resident/`, feature `resident`), field
evaluation and Burn lowering into `numen` (`eval.rs`, `lower_burn.rs`), and
its forces into `seiche` (`crates/conatus/seiche/src/tensor_forces.rs`).
Read "Quint" below (§2,
§3, §5 and the implementation order's items 4 and 6) as those owners.
`conatus-brick` is `modulus` since `33f9b6b6` (2026-08-28).

The dependency direction is runtime to implementation only. Product crates
depend on Conatus. Conatus must not depend on a game.

## Implemented foundation

`crates/conatus/conatus` now supplies:

- stable generational body and collider identities;
- fixed, dynamic, and position/velocity-kinematic 3D bodies;
- compound colliders with sphere, box, capsule, cylinder, and sparse voxel
  shapes;
- materials, collision layers, sensors, gravity, damping, CCD, force, torque,
  and impulse operations;
- ray casts and shape-overlap queries using Conatus identities;
- contact and sensor entry/exit events;
- in-place sparse voxel collision edits with revisions;
- configurable kinematic character movement with wall sliding, slope limits,
  ground snap, and autostep;
- a drift-free fixed-step engine clock with explicit catch-up policy;
- ordered ingest, field, before/after-physics, materialization, and publish
  system phases;
- typed shared resources and deterministic registration order;
- a serializable structural command buffer applied between phases, with
  correlated command results;
- current-step physics changes and interaction events available to
  after-physics systems;
- frame changes containing final body transforms and removals, including host
  frames on which simulation takes zero steps;
- effective sparse voxel edit streams for materialization and render systems;
- serializable body, collider, voxel, filter, and character configuration.

Rapier types stay private at the public API. That is insulation, not yet
interchangeability: the initial `BodyWorld` directly embedded Rapier handles,
lowerings, queries, character movement, voxel mutation, stepping, and event
translation in one implementation. The 2026-08-24 integrity pass first makes
that private implementation boundary structural. A future backend must still
prove which game-facing behaviors it supports; unsupported behavior may not be
silently approximated.

### Private backend integrity pass (complete 2026-08-24)

Keep Conatus identities, generations, revision order, dirty publication, and
normalized events as the stable spatial mechanics. Move Rapier-specific world
state, handles, conversions, queries, character controller, voxel shape edits,
and stepping behind a crate-private implementation module. This pass adds no
public backend trait, backend selector, or Nexus dependency.

Done when the existing public API and all behavior tests remain unchanged,
Rapier imports and handles are confined to the private implementation, and
warnings-denied checks pass. This proves a code boundary only. Nexus earns a
backend seat later through an isolated lifecycle/query receipt and the exact
host-device receipt; it does not inherit one from opacity alone.

Complete at Mere commit `339e8567`: six `conatus-voxel` tests, eighteen
Conatus unit tests, the cross-package voxel-collider integration test, and
warnings-denied Clippy passed.

## Runtime growth

### 1. Runtime and systems (foundation implemented)

The ordered spatial schedule now has concrete phases:

1. run ingest adapters over already-admitted spatial work;
2. evaluate fields;
3. run before-physics spatial systems;
4. advance tactile physics;
5. run after-physics spatial systems;
6. materialize voxel and geometry changes;
7. publish derived spatial changes;

Systems receive typed resources, current-step output after physics, and a
command buffer. Structural mutations
apply between phases, so scripts and parallel systems cannot invalidate a
live iteration. Tick-local events and durable game facts remain different
types.

Three corrections (2026-08-23) narrow this machinery to spatial work:

**Phase names.** On 2026-08-23, `Input` became `Ingest`, `Gameplay` merged
into the existing `BeforePhysics` phase, and `Prepare` became `Publish`.
The public schedule now names only spatial work and does not imply that
Conatus owns user input, rules, or rendering.

**Command trust boundary.** Scripts and peers submit product intents; only
authorized product code — the profile and its registered systems — lowers
accepted consequences into local `BodyCommand`s. Raw remote `BodyCommand`s
leave the architecture, and `command.rs`'s doc line advertising the command
vocabulary to "scripts and remote inputs" is corrected with the rename. The
first product profile retains the request provenance alongside its admitted
intent; only the product decides whether the request is allowed. Conatus does
not invent a universal provenance field before that consumer names what it
needs.

**What a source means stays profile-owned; its body table will be a Conatus
module.** `BodyId` is generational and runtime-only, and `BodyDesc`
deliberately carries no durable source reference. One durable product
source may materialize as several bodies, scene instances, resident slots,
and audio voices, so what a source is, when it binds, and what it becomes
belong to the runtime profile, not inside `BodyDesc`. Sceno's `SourceRef`
is the pattern reference, not automatically the universal type: it belongs
to semantic scenes and lacks revision and materialization information.
Isometry's retired accepted-map profile and Mesocosm's `TactileWorld`
provided the two compared consumers. Mark ruled the common minimum a
Conatus module generic over the product's key, documented now and built
when he says (wing rulings 346 and 348). Until then each product keeps its
own table. The dated 2026-08-23 ownership correction above remains history;
this paragraph refines the mechanism's home without moving source meaning.

#### Body bindings (ruled 2026-09-26; documented, not built)

The comparison and complete adoption conditions are in
`isometry/mesocosm/design_docs/2026-09-26_body_binding_plan.md`, §2 and §6.
Rulings 346 to 352 are recorded in
`isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`.
This section carries Appendix A's module shape into its owning plan;
it does not open implementation. Mesocosm keeps its `TactileWorld` table;
the VTT has none after retiring `isometry-runtime`.

When authorized and built, the module works as follows:

- **Bodies only, on the caller's world** (349). Calls borrow the caller's
  `BodyWorld`, available through `Engine::bodies_mut`, so bound bodies and
  T2's terrain collider share one world. The module owns no terrain, clock,
  or durable state, and never serializes a `BodyId`.
- **The table** (348). A product key maps to one `BodyId` and the caller's
  shape revision, with a corresponding reverse map. The product supplies
  map-qualified token keys or critter identities. Several bodies for one
  source use a compound key such as `(source, part)`.
- **Accepted state, whole or per key** (350). Reconcile accepts a complete
  set, spawning new keys, moving changed poses, and despawning absent keys.
  Reconcile on an empty table is the cold rebuild. Per-key set and remove
  apply incremental changes; both routes must agree. Only authorized
  product code calls these operations after accepting state; no intent
  enters the module.
- **Stable identity on movement** (351). A new pose preserves `BodyId`;
  changing the caller's shape revision respawns the body.
- **Refusals before mutation.** Duplicate keys and a bound body removed
  behind the module are refused before any world call. If a later Conatus
  operation fails partway through applying an accepted set, the table must
  still name exactly the bodies actually held by the world.
- **Queries stay Conatus's.** The reverse map names bodies returned by
  rays, overlaps, movement and frame updates. Unbound bodies, including
  terrain, have no product key. The planned query-refresh call (352) makes
  topology changes query-visible without a settle step. Pointer picks on
  a drawn frame belong to isometer (347).

Source meaning, admission, collider lowering, scene instances, resident
slots and audio voices remain product-owned. Eponym's per-move query world
can use a cold reconcile; that is an adoption target, not a current shared
consumer receipt.

**Implementation done-conditions, still gated by ruling 346.** The module
names no product type and adds no Conatus dependency; it meets the body
binding plan's size and failure conditions. Seeded accepted sets and edits
must give equivalent cold and incremental keys, shapes and transforms.
Removal/recreation must invalidate the old body and reverse lookup; moved
bodies retain their IDs; duplicate keys and lost bodies refuse safely.
Product adoption preserves caller lowering and uses the shared query-refresh
API after its own verification and product repin. Mesocosm's critter half
may then adopt the table; its terrain half belongs to T2. The VTT adopts
only when accepted tokens actually require Conatus bodies, using the scene
board's coordinate convention rather than reviving the retired runtime.

The remaining runtime work is parallel system access declarations, enforcing
the intent-lowering command boundary at the first product profile, and the
lean spatial-frame resource (§4). A game can already register spatial
systems, insert resources, queue structural commands, and advance without
writing a physics loop or borrowing Rapier types.

### 2. Voxel world

Promote the generic parts already present across Mesocosm and Quint:

- sparse chunk addressing and stable chunk ids;
- exact occupancy/material planes with per-chunk revision;
- dirty brick/region tracking and bounded edit batches;
- derived collision, surface, SDF, light, navigation, and mesh products;
- dependency stamps so each product updates only from changed source regions;
- streaming budgets and explicit residency states;
- body volumes using the same plane and product vocabulary as ground volumes.

The first CPU mechanics slice landed as the Rapier-free `conatus-voxel`
package (now `nisus`): Euclidean world cell addressing; dense opaque chunks in the incumbent
Mesocosm Y/Z/X order; validated serialization; revision-gated, caller-bounded
patch batches; effective changes; disposable dirty boxes; and lowering of
material changes into backend-neutral occupancy edits. The full `conatus`
runtime re-exports this vocabulary and lowers accepted occupancy edits into its
voxel collider. Product chunk identity, material meaning, admission, and
durable authority remain outside either package. Mesocosm's `Ground` and
Quint's `ResidentChunk` are unchanged.

Games define what a material or voxel means. Conatus owns storage,
revision, locality, and derived spatial products.

Feature complete when terrain edits, body-volume edits, collision, queries,
and mesh/SDF preparation use one chunk/revision path and game crates carry no
second voxel cache protocol.

#### T2. One edit, all spatial consumers (planned; after pre.4)

**Status, 2026-09-28:** the shape below carries wing rulings 330 to 335 and
363 into this owning section. Nisus currently supplies chunk value
mechanics; this world-store and consumer-revision path is not built.
Implementation waits for acceptance of the pre.4 migration (363), whose
Conatus changes must land before T2. No migration acceptance is inferred
from this documentation update. The source rulings live in
`isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`;
`isometry/eponym/design_docs/2026-09-09_functional_loops_plan.md` retains the
product receipt and adoption dependencies.

*Annotation, 2026-10-06 (S14 pass):* main moved to pre.4 on 2026-10-05,
when S16 merged the pre.4 branch at `cec0b3a4` under ruling 557, "Promote,
then handoffs" (burn migration plan §13.44). Whether that is the acceptance
ruling 363 waits for is not recorded here.

*Ruled 2026-10-06:* asked whether ruling 557's promotion, together with Knot's
and djinn's repins onto pre.4, counts as 363's acceptance (options: yes; no,
not until Isometry repins; ask the wing session), Mark answered **"Yes,
however, 0.22 also dropped today."** *Follows:* T2's pre.4 condition is met. T2
still needs its own lane and Isometry's pin. Burn 0.22.0 stable was published
2026-10-06T18:46Z and CubeCL 0.11.0 at 15:59Z (crates.io), so the next Burn
step is pre.4 → 0.22.0, which goes to Mark as its own assessment (burn plan).

1. **Nisus becomes the voxel world store** (330): a chunk map, world
   revision, revision log and additive writes extend its revision-gated
   chunk patches. `Ground` becomes a thin product layer over that store or
   retires. Existing saves and hashes require an explicit migration or
   preservation of the existing wire format. Product material meaning and
   admission remain product-owned.
2. **Each consumer reads a revision log** (331). Changes are recorded at
   8³-brick grain. Each consumer retains its own last-read revision and
   reads changes since then; a consumer older than retained history rebuilds
   from the current store. The log replaces a single destructive dirty
   queue as the means of serving several consumers.
3. **Conatus carries the source stamp** (332). Voxel edits take an optional
   source stamp which spatial queries can report. The product holds
   dependent simulation until required colliders reach the source revision.
   That barrier stays product-local until a second consumer proves a shared
   mechanism. Conatus's current internal revision counter is not evidence
   that a collider reflects a particular source revision.
4. **Navigation remains a stamped search per query** (333). Each result
   records the terrain revision it read. A cached walkability product is
   deferred until measured query cost warrants it.
5. **Verify inside Mere, then adopt in Mesocosm** (334). The first integration
   exercises the Nisus store, Conatus collider updates and modulus refresh.
   After a product repin, Mesocosm proves both carving and additive writes
   update collider queries, routing and render slots at one source revision.
   Eponym's crossing/adoption follows its own E2 dependencies; T2 does not
   open the broader T1 material transaction or T3 work-consequence lanes.

**Done-conditions.** After the migration gate is met, accepted patches
advance the shared source revision and refused stale patches mutate nothing.
Independent consumers neither consume each other's changes nor silently
miss edits; an out-of-retention consumer rebuilds correctly. The Mere
integration verifies removal and addition across storage, collider queries
and modulus, including source stamps. Mesocosm's adoption verifies render,
route and collision agreement and holds dependent simulation while a
required projection is behind. Its old terrain mirror/dirty protocol is
then removed or reduced to the thin layer, with save compatibility recorded.
Body binding may share this `BodyWorld` but is a separate module and gate.

### 3. Resident spatial world

Join the CPU body world to Quint's resident allocation model:

- stable GPU slots with generations and free-list reuse;
- padded 3D position, rotation, velocity, force, and flags planes;
- changed-slot uploads for the Rapier tier;
- direct resident advancement for field/particle tiers;
- small reductions and accepted deltas as the only routine readback;
- leases with allocation epoch, source revision, shape, units, and valid read
  interval.

The first position-plane slice is complete at Mere commit `c382e734` and
Isometry commit `15f5da2`. Quint can validate and publish several disjoint
ranges under one stamp. Isometry uses that mechanism to project an accepted
`IsometrySpatialFrame` into one fixed-capacity `[x, y, z, occupied]` plane.
The product owns capacity, coordinates, source bindings, generation handling,
and tenant selection. Conatus's existing `FrameUpdate` remains unchanged.
This disposable product projection does not settle allocation ownership for
state advanced directly on the GPU, or establish a shared frame or lease.

*Annotation, 2026-10-06 (S14 pass; relayed, not verified here):* the S14
pass's record (`support/doc-audit/d2/batch_46_s14_phase_b8.md`) and its
coordinator relay that this slice's Isometry half (`15f5da2`) and §4's
marker tenant (`7d45c40`) lived in Isometry's `isometry-runtime` crate,
which Isometry retired at `73a31409` on 2026-09-27 under wing ruling 299.
Isometry's repository was not read for this annotation. Both are historical
receipts, not live code; §1 already records the crate's retirement. Mere's
half, `c382e734`, is on main, its resident code now in `conatus::resident`.

Burn/CubeCL handles dense fields and authored kernels. Khal/rust-gpu artifacts
are adopted where their explicit spatial algorithms are useful. Tool choice
follows the operation. Allocation ownership follows advanced state: Conatus
owns the allocations whose state it advances, the profile orders passes on
the shared device and queue (the host-conducts ruling, spatial compute plan
2026-08-13), and Netrender's tenancy seam stays the device seam. Conatus is
not the global allocator for ESP, Quint, Renderling, or future subsystems.

Feature complete when CPU tactile bodies and GPU field bodies can be joined
through one versioned profile-local spatial view and coexist in one spatial
world. Its cross-product vocabulary becomes stable only after a second game
consumes it.

### 4. Spatial frame

Publish a lean spatial frame: transforms addressable by runtime identity,
removals, contacts, activity, sleeping and residency changes, voxel and
geometry revisions, and resident products. No cameras, no lights, no sprites,
no visibility or presentation policy: those belong to the product's rendering
profile, which selects and configures tenants — Scenograph lanes for semantic
2D, Renderling for 3D bodies, brick DDA for live volumes (owned by
`conatus-brick`, currently on the `codex/conatus-brick-lift` branch and
consumed by pinned rev), several composed by Netrender.

The first realized adapter is Isometry's product-local, fixed-isometric body
marker tenant at commit `7d45c40`. It binds the stamped Quint position
suballocation directly, projects through configurable basis and appearance
settings, renders into its own same-device texture, and gives Netrender an
explicit external-composition boundary. Netrender learns neither Conatus body
semantics nor Quint allocation policy.
*Annotation, 2026-10-06 (S14 pass; relayed):* this tenant was retired with
`isometry-runtime` (§3's annotation) and is a historical receipt.

Renderling remains a candidate for a later 3D portion through another
profile-owned adapter. It was not pulled into the 2D proof because its current
stage cannot attach the external resident plane without a sidecar pass. Mere's
spatial renderer may consume a later common frame if another product proves
one. Netrender receives each tenant's frame entry and composes them on the
host's device and queue.

The frame vocabulary is a cross-product contract, so its shared form stays
provisional until a second game consumes it. Until then it is the Isometry
profile's seam, kept neutral in shape.

Feature complete when a product selects render tenants and presentation
settings in its profile while body transforms, activity, and geometry
revisions continue to originate in Conatus.

### 5. Procedural and field systems

Keep Numen and Quint independently callable, including by Seiche and product
profiles. Add Conatus adapters only for field effects on state Conatus
advances:

- scalar/vector field sampling over Conatus bodies and volumes;
- force and material couplings applied to Conatus state;
- particle and fluid systems whose state Conatus advances;
- SDF composition and extraction for Conatus geometry;
- spatial trees, neighborhood queries, and large-body broad phase.

Generic procedural algorithms may live in shared crates. Terrain, body,
vegetation, and scatter recipes remain product content. Feature complete when
a profile can select a field adapter for Conatus while Seiche and another
consumer continue using the same Numen definitions and Quint evaluation
without depending on Conatus.

### 6. Scripting and content

Expose spatial queries, events, system parameters, body/voxel descriptors,
and spatial body or voxel recipes to the script host. Scripts submit product
intents; authorized product code lowers accepted consequences into commands.
Scripts cannot obtain raw backend or GPU handles, and they do not receive the
raw command buffer. Content catalogs select and combine registered features
instead of growing fifty bespoke enums per category.

Feature complete when a data/script-defined rule can spawn and alter bodies,
edit voxel regions, query space, react to interactions, and configure fields
through product intents that lower into the same command vocabulary native
product systems use.

## Immediate implementation order

1. Make the current private Rapier implementation a real internal boundary;
   keep backend selection private until a named product workload forces a
   second implementation.
   *Annotation, 2026-10-06 (S14 pass):* done 2026-08-24 at `339e8567`, the
   private backend integrity pass above; backend selection stays private.
2. Keep extending `conatus` as the shared spatial package; `seiche` stays the
   2D graph specialist rather than the 2D graph API becoming the 3D core or
   being forced through it.
3. Adopt the generic voxel chunk/revision/dirty-region mechanics in one product
   without moving product identity or authority, then feed accepted occupancy
   changes into the voxel collider already implemented.
4. Publish versioned profile-local resident body/chunk views through Quint
   allocations. The first body-position view is complete; rotation, velocity,
   forces, flags, voxel/chunk joins, and direct-GPU advancement remain open.
5. The first profile-owned Netrender tenant adapter is complete in Isometry:
   it realizes stamped resident positions as fixed-isometric body markers.
   Add a Renderling or other 3D adapter only when a product lens demands it,
   and keep shared frame vocabulary provisional until a second game challenges
   it.
   *Annotation, 2026-10-06 (S14 pass; relayed):* the body-position view in
   item 4 and this tenant were Isometry's, retired with `isometry-runtime`
   (§3's annotation); Quint's allocations are `conatus::resident`'s (Existing
   pieces).
6. Add optional field adapters and spatial scripting against the shared
   resources without making Numen or Quint depend on Conatus.

New work belongs in a product only when its meaning is genuinely product
specific. The extraction rule is split (ruled 2026-08-23):

- **Mechanics may move early.** A reusable implementation whose authority is
  already settled — collision algorithms, voxel revision machinery, spatial
  queries — enters its natural shared crate after one forcing consumer, as
  it is written, rather than after another probe duplicates it.
- **Contracts wait for the second consumer.** Public contracts governing
  orchestration, identity, authority, device ownership, or cross-product
  frames — the conductor, source bindings, the spatial frame, shared trigger
  vocabulary, resident lease contracts — stay provisional until a second
  game proves them. The first implementation uses a neutral-shaped seam in
  its profile; the second consumer tests that shape before it becomes stack
  law.

The body-table comparison in §1 has satisfied that comparison requirement;
its implementation remains separately gated by ruling 346. It does not
promote a universal source identity, conductor or projection barrier.

This reconciles the plan with the wing's two-consumer law (mesocosm
`CLAUDE.md`): no deliberate duplication of machinery, and no cross-product
contract declared in advance.

## Progress (2026-08-23 product adoption pass)

- Conatus's corrected spatial-runtime foundation and first voxel mechanics
  landed at Mere commit `5767563c` with 24 tests and warnings-denied Clippy.
- Mesocosm became the first tracked mechanics consumer locally at `b112931`:
  `Ground` remains authoritative while a product adapter uses `VoxelChunk`
  patch and occupancy machinery, with replay, refusal, snapshot-silence, and
  unchanged-source receipts. Its divergent main and concurrent dirty lane keep
  remote integration open.
- Isometry's first profile landed product-side at `303e347` in
  `isometry-runtime`: accepted map events drive an event-cadenced, zero-step
  Conatus projection with map-qualified source bindings. Host wiring remains
  gated by Isometry's active protocol work and Genet host migration.
- The cross-product ownership and receipt queue now lives in the
  [runtime composition acceptance plan](2026-08-23_runtime_composition_acceptance_plan.md).
  No conductor, source-binding, spatial-frame, trigger, or resident-lease
  contract was promoted in this pass.

## Progress (2026-08-24 private-boundary pass)

- `BodyWorld` now retains Conatus identity, revision, and publication
  bookkeeping while all Rapier state, handles, conversions, queries, character
  movement, voxel mutation, stepping, and interaction normalization live in a
  crate-private implementation module. This is an internal boundary, not a
  public backend ecosystem.
- Generic chunk, patch, dirty-region, serialization, and occupancy-edit
  mechanics moved into `conatus-voxel`; `conatus` re-exports the same public
  vocabulary. Mesocosm now names the narrow package directly, so its
  `GroundVoxelProfile` does not acquire Rapier merely to maintain a disposable
  chunk view.
- The isolated Nexus probe resolves Netrender, Vello, Khal, and Nexus onto one
  wgpu-30 row after restating Netrender's temporary Vello patch. Its Windows
  build reaches `nexus_rbd3d`, then the Khal build script's `cargo-gpu 0.1.0`
  invocation fails while removing `Cargo.lock`. No Nexus kernel executed, so
  Nexus remains outside Conatus and shared buffer ownership remains unproven.
  *Annotation (2026-10-01):* diagnosed and cleared after Mark ruled "Fix now"
  (physics catalog plan §5). The failure was a race, not a missing-file bug:
  two Nexus build scripts (vortx's and `nexus_rbd2d`'s) each run `cargo gpu
  build`, and cargo-gpu 0.1.0 has no lock around its shared codegen install,
  so one process's `Cargo.lock` removal or `target\` cleanup lands under the
  other (os errors 2, 145 and 3; all three cold concurrent runs failed). The
  rust-gpu fork's cargo-gpu 0.10.0-alpha.1 already takes a `FileLock` on the
  install; with it unpatched, the same cold runs passed 2 of 2, `nexus_rbd2d`'s
  five GPU radix-sort tests passed, and `nexus_rbd3d`'s `test_stacks_1_tiny`
  ran 250 GPU steps on the RTX 4060 (Vulkan). Logs:
  `Code/testing/nexus-build/`. No patch was made; whether to patch anyway,
  replace the installed 0.1.0 binary, and pin the codegen version are open.
  This proves Nexus kernels execute here; shared buffer ownership with the
  host's device is still unproven.
  Ruled the same day:
  - **The fork patch.** Mark chose no patch ("1"), and added: "also make sure
    we're up to date for, rust-gpu, renderling, and nexus. they develop fast.
    let's get what we can from their respective upstreams". At that check,
    nexus was 2 commits behind upstream main (`1cfbd76`), the rust-gpu fork
    branch 27 behind (`0a9d096f32`, the v0.10.0 release) with our two
    version-gate commits on top, renderling 1 behind with our four wgpu-30
    commits plus 26 uncommitted files last touched 2026-09-15, and the
    standalone cargo-gpu archived upstream (merged into rust-gpu).
  - **The installed cargo-gpu.** "Replace, after checking renderling": build
    renderling's shaders with the new cargo-gpu first; replace
    `~/.cargo/bin/cargo-gpu` 0.1.0 only if they pass.
  - **The codegen version.** Told the cache held `rustc_codegen_spirv` 0.10.0
    while Nexus used spirv-std 0.10.0-alpha.1, Mark asked "wait. why aren't
    we on the most up to date...? sure, 3": move Nexus to the 0.10.0 line
    (upstream main) rather than pinning the codegen back. The answer to his
    question: the forks were last synced in August and September, and
    spirv-std 0.10.0 was released on 2026-10-01, so alpha.1 was current
    until that day.
  - **The sync, carried out and ruled further.** Nexus fast-forwarded to
    upstream `1cfbd76`; spirv-std now resolves to 0.10.0 and the codegen to
    0.10.0, and both GPU proofs pass again (`Code/testing/fork-sync/`).
    rust-gpu's version-gate bug has no upstream fix but is latent (upstream
    pins nightly-2026-07-03, 1.98). Mark chose "cargo-gpu 0.10.0 from
    crates.io": the binary comes from the published release, the fork branch
    stays as the record, and the standalone `crates/cargo-gpu` fork is no
    longer used. The alternatives were building from tag v0.10.0, rebasing
    and carrying the patch, or rebasing and filing it upstream. Renderling's
    one upstream commit (`46bf54c`, manual chapters and a
    `Stage::tonemapping()` accessor): "Merge upstream in" to
    `mark-ik/wgpu-30`, keeping our four commits' hashes. Its 26 uncommitted
    files, rustfmt output from the repo's nightly-only options with one
    reflowed expression as the only non-comment change: "Commit as a
    formatting commit" first. The three codegen caches Nexus no longer uses:
    "Delete after renderling's check".
  - **Renderling's check, and the binary** (2026-10-02). Renderling's
    formatting commit (`e14b737`) and upstream merge (`260e2c2`) landed on
    `mark-ik/wgpu-30`. Our branch's shaders build with neither cargo-gpu
    0.1.0 nor 0.10.0: `naga` 30, a build-dependency since our wgpu-30 port,
    needs rustc 1.87, and the shader toolchain is nightly-2025-02-16 (1.86).
    Upstream renderling (`46bf54c`) builds with 0.10.0, all 45 `.spv`
    byte-identical to the committed ones, which our branch shares; renderling's
    95 library tests pass run serially. Mark chose "Replace now, fix the
    branch separately": cargo-gpu 0.10.0 becomes the installed default and
    the three unused codegen caches go. For the branch, "Port shaders to
    rust-gpu 0.10": spirv-std 0.10 and nightly 1.98, one toolchain with
    Nexus. The alternatives were an older naga for build.rs, gating build.rs
    off for shader builds, or leaving it until a shader changes.
  - **The renderling port's blocker** (2026-10-02). cargo-gpu 0.10.0 is now
    the installed default and the unused caches are gone. A throwaway trial
    ported renderling's shaders to spirv-std 0.10 with version and toolchain
    moves only: 95 of 95 tests, WGSL byte-identical, 44 of 45 `.spv` rebuilt.
    It is blocked by crabslab (`crates/crabslab`, `mark-ik/wgpu-30`
    `a1ffc17`), which requires `spirv-std = "0.9.0"`; eponym builds against
    the live crabslab and renderling checkouts. Mark chose "Sync crabslab
    upstream first" (13 behind, upstream `f990323` on wgpu 26 and spirv-std
    git `b3eda4df`); the alternatives were widening the range on
    `mark-ik/wgpu-30`, or a new 0.10-only branch. The port runs in the
    **live checkout** (eponym follows it), the rebuilt `.spv` are
    **committed**, and renderling's build.rs running `cargo +nightly fmt` on
    every host build is **recorded only**. *Reading, not ruled:* the crabslab
    sync merges upstream in, as renderling's did.
    *Reopened (2026-10-02):* the sync is not mechanical. Upstream's 13
    commits rewrite craballoc (0.4.0 / crabslab 0.7.0, unreleased), deleting
    `slab.rs`, `value.rs` and `wgpu_slab.rs`, the files our three wgpu-30
    commits port; a merge conflicts in five files, three modify/delete.
    Upstream is on wgpu 26 and spirv-std git `b3eda4df` (still 0.9.0). Both
    upstream renderling and ours require craballoc 0.3.1 / crabslab 0.6.6 and
    use the deleted API, so a merged checkout would stop matching their patch
    and Cargo would silently take crates.io 0.3.1 on wgpu 26. Asked how
    crabslab should move, Mark said: "Full adoption, or consider what would
    suit the stack best… how could we make renderling the ideal for us?"
    Open: an assessment of renderling's role in the stack comes back to him
    first.
    *Assessed and answered (2026-10-02):* renderling is already ruled out by
    the presentation plan's L7 (`isometry/mesocosm/design_docs/
    2026-09-11_orthographic_voxel_presentation_plan.md:418-426`, done-condition
    unmet), ruling 27 keeps the renderer swappable (kiss3d first, renderling
    "far later"), and ruling 442's recommendation retires it before the mode
    host; only `eponym-client`'s `Tenant` and two probes use it. Upstream
    craballoc 0.4 is unadopted by renderling and superseded by crabslab's
    `feat/wgsl-rs` (crabslab 1.0 / craballoc 0.5, wgpu 28). Mark's answers:
    - On reopening L7: "Hmm. Kiss is the straightforward choice for both 2d
      and 3d. Renderling, the five things we'd get from it, how's that
      compare to kiss, or other alternative prospective pieces of game engine
      that would compose into the stack? Consider that in all cases, I am
      willing to reshape a good candidate into an excellent stack
      component/module/crate; i don't mind renderling, kiss, or another option
      as long as they compose well and improve the whole stack with their
      capabilities. Whether that's rendering, entity management systems, etc.
      etc. i don't even mind measuring both, or considering wgsl-rs or
      rust-gpu or whatever". Open: a comparative assessment of candidate
      engine components comes back to him.
    - Allocator and shader lane, if renderling is kept: "Residency via
      conatus/CubeCL": slab residency is replaced by conatus buffers bound
      directly, one allocator.
    - Harvesting renderling's lighting: "Compare to what would suit the stack
      and wing": folded into the comparative assessment.
    - The R2 receipt: "Re-prove in isometer-render": `isometer-render` binds
      conatus's CubeCL buffers directly, testing whether the copy and the
      second allocator disappear.
    *Engine-component comparison ruled (2026-10-02).* kiss3d 0.46 (wgpu 30,
    WGSL, BSD-3) covers renderling's five features (shadow maps, clustered
    lights, PBR/IBL, glTF animation, skinning with morphs on web) plus SSAO,
    OIT, transmission and 2D lighting; it takes the host's device but keeps a
    thread-local `Context` singleton (`kiss3d-0.46.0/src/context/context.rs:12`)
    and wants a window. Renderling's edge is GPU-driven slab instancing. The
    wing's terrain is traced and its bodies rasterised, joined by depth (L3).
    - **Renderer tenant:** "kiss3d, reshaped": an explicit context handle in
      place of the thread-local, a render-into-caller-targets entry with no
      window, the tracer's depth as a pre-pass, its shadow atlas and light
      buffer exported. The alternatives were growing isometer-render with
      both as donors, re-adopting renderling, or measuring both first.
    - **Lighting:** "Stack-owned light/environment block": sun and
      day/night from sim fields, the point-light list and the water field
      live in the scene contract, read by the tracer and the rasteriser; the
      renderer exports shadow atlas, light buffer and depth. The alternative
      let the renderer own lighting.
    - **Shader lane:** "WGSL/WESL for raster, CubeCL for compute", amending
      the 2026-08-16 "author in CubeCL, the brick renderer included" line to
      match what ships (all five wing render shaders are WGSL); rust-gpu stays
      for Nexus-derived compute. The alternatives were wgsl-rs, or rust-gpu
      0.10 for raster.
    - **The renderling fork:** "Archive the fork now". The renderling port to
      rust-gpu 0.10 and the crabslab sync stop. *Reading, not ruled:* eponym
      still path-depends on the live checkouts, so the archive move waits on
      L7 or eponym breaks; put back to Mark.
    These are games-wing decisions; their canonical home is the wing design
    record in `isometry/mesocosm`, where they have not yet been carried.
    Further, the same day:
    - **Archive timing:** "L7 first, then archive": eponym-client's Tenant
      and lighting move off renderling, its two probes are archived and the
      patch rows dropped, then both forks move to `archive/`. This resolves
      the reading above.
    - **ECS:** "Not bevy, but can we compare the potential of the other
      two?": hecs and shipyard are being compared; open.
    - **2D:** "vello for documents, kiss3d 2D for lit games": kiss3d's 2D
      only behind the scene contract for a lit 2D game. The alternative was
      vello only.
    - **Where they live:** "You can do 1, but let the wing session know":
      they are carried into the wing design record as numbered rulings, with
      the "Isometric game engine architecture" session told first; it was
      mid-round (rulings 457 to 466 that night), so the carry waits on its
      reply to avoid a numbering collision.
    *Carried (2026-10-02):* the engine rulings are wing record rulings 471
    to 476 (isometry `36f6f69`), and the ECS, ruled the same day, is 481 to
    484 (isometry `b4a0387`): hecs stays; the mode host and armillary
    schedule, the ECS is storage; the projection's diff comes from the
    record's receipts; a boundary crate in Mere holds the ECS. The wing
    record is their authority.
  - **The logged token.** Four build logs captured this session's
    environment, including its messaging token and account IDs; redacted on
    Mark's choice.

## Progress (2026-08-25 resident-position pass)

- Quint commit `c382e734` added an atomic sparse batch-patch mechanism to
  `ResidentChunk`. Its real-adapter receipt changes nonadjacent rows under one
  advancing stamp, retains the allocation, and proves malformed batches write
  nothing.
- Isometry commit `15f5da2` added the first product-local consumer: accepted
  Conatus body changes populate a retained position plane, silent frames do
  not write, capacity refusal leaves the allocation untouched, and same-slot
  generation reuse remains distinct in the product binding. The accepted
  `MapDocument` stays authoritative and unchanged.
- Shared frame, source, lease, and conductor contracts remain provisional.
  The next critical slice is the product-owned renderer tenant adapter; host
  construction still waits for Isometry's protocol and Genet migration gates.

## Progress (2026-08-25 renderer-tenant pass)

- Isometry commit `7d45c40` added the first product-owned resident renderer
  tenant. Its production path reads the stamped Quint storage suballocation
  directly, draws configurable fixed-isometric markers into a tenant texture,
  and exposes that texture to Netrender at an explicit scene boundary.
- The real-device receipt proves scene content below and above the tenant,
  silent-frame skipping, stable-allocation moves, stale-view refusal, removal,
  generation-aware slot reuse, and capacity and target-limit refusal. Default
  profile tests remain GPU-free.
- This closes the first renderer-adapter slice, not host adoption or a shared
  tenant contract. Isometry desktop construction still waits for protocol H2
  and Genet migration. Renderling remains a later 3D candidate, and shared
  frame, source, lease, tenant, and conductor vocabulary still waits for a
  second product.


## Consolidation map (ruled 2026-08-28, Mark; amended the same day from a three-engines sketch)

The consolidation goal, stated plainly: functional, modular, nonredundant
components, consolidated by decomposing incumbents into owned organs behind
stable vocabularies — never by umbrella crates. The map:

- **Realms own truth.** Chartulary is the graph realm; the Ground-pattern
  with `nisus` is the voxel realm; esp's corpora are the semantic realm.
  Each realm owns authority, revision, and edit mechanics, and nothing else
  does. Analytic products form **derived realms** beside authority —
  revisioned, disposable, refusal-gated, the `GroundVoxelProfile` and
  tactile-advice shape — never a second authority.
- **Two engines operate over realms.** The **projection engine** shows
  realms, primary or derived: sceno/scenomise, the graph canvas (a graph is
  a projection of data — the earlier sketch's "graph engine" dissolves here
  as chartulary's lens), `modulus` with the product tracers over it,
  Renderling tenancy, Netrender composition. The **inference engine**
  (esp's semantic lane plus quint's Burn lanes) couples twice and only
  twice: **analytically**, reading realms into derived data; and
  **generatively**, writing proposals into realms under
  propose-constrain-commit with authority disposing (the resident-ground
  receipt and Mesocosm's B1 bounded policy are the standing proofs at two
  scales). Inference never renders; a lens that "depends on the inferences
  upon the data" is just projection over a derived realm, in games and in
  turnstone alike.
- **Physics is a stratified capacity, not an engine.** In-projection
  dynamics (`seiche` over layout space; quint's `z_field` at the 2.5D
  rung), in-realm dynamics (`conatus`, and the decomposed Nexus backend
  when its consumer fires), and fields serving either altitude (quint).
- **The host grows out of genet/cambium.** The game runtime shell is the
  same lane as Isometry's pending Genet host migration; the hand-rolled
  winit hosts in the vessel receipts retire into it.

**Nexus is decomposed, never adopted.** The runtime's body vocabulary
already keeps its backend private precisely so "a later Nexus or
resident-GPU backend can replace that machinery" — the ruling sharpens
that seam: when the first vessel gate needs dynamic bodies (Paredros F5
material life and F7 danger are the expected pulls), Nexus's useful parts
arrive as a Conatus backend behind the same `BodyWorld` vocabulary, on the
one-device CubeCL/wgpu lane the mesocosm R2 receipt proved. Nexus never
becomes a peer engine with its own vocabulary. This retires both standing
hazards at once: the upstream Windows shader-build blocker stops mattering
(only the kernels that serve this stack are taken), and the Parry-`Voxels`
admission gap closes from our side (its solver meets the world through
Conatus's voxel colliders rather than its own geometry path). The missing
quadrant this fills is GPU *dynamics*: the family already has GPU fields
and GPU residency, and its rigid bodies are CPU-only until then. License
diligence on the upstream source happens at decomposition time, before any
kernel is taken.
## Progress (2026-08-26 brick-traversal ownership pass)

- `conatus-brick` now owns the deterministic sparse pointer/atlas layout, its
  explicit projection revision, the exact GPU trace-space layout, and the
  camera-neutral WGSL DDA. It accepts selected brick bytes and caller-supplied
  rays; it owns no Ground, camera, material, lighting, body, frame, or lease
  vocabulary.
- Mesocosm's former implementation is reduced to a Ground source adapter and
  product presentation shader. Paredros has its own source adapter and is the
  second tracked product consumer. The two compile the same shared WGSL under
  orthographic and perspective cameras.
- Quint remains the owner of resident allocations. The next join is therefore
  incremental `ResidentChunk` publication into this ABI, not moving the DDA
  into Quint or inventing a universal voxel renderer. Raymarch depth and
  Renderling occlusion remain the next presentation boundary.

## Progress (2026-09-26 brick-atlas pass)

Two changes to `modulus`, from Isometry's board-paging lane and ruled by Mark
in the wing design record (rulings 288 and 289,
`repos/isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`).

- **Finding: a shrinking retarget broke the kept bricks' reads.** A retarget
  keeps each retained brick's atlas slot, so after one that shrinks the
  selection a kept brick can hold a slot past the number of resident keys.
  `atlas_slot_origin` refused any slot above that count, which looked like a
  bound only because `from_keys` packs slots densely. For such a brick
  `material_at` read air under ground the tracer still drew, so a pick passed
  through it; `slot_texels` returned `None`; and `refresh` panicked in
  `write_slot`. Loaded keys take the lowest free slots, so only retained
  bricks could trip it. Fixed in `406aafb2`: slots are bounded by
  `capacity()`, the geometric question every caller asks. The regression
  tests in `crates/conatus/modulus/src/tests.rs` fail on the old bound: the
  kept brick's refresh, a pick checked voxel for voxel against a mirror of
  the shader's `brick_material_at`, and every slot up to capacity owning its
  own box.
- **The atlas is sized to the card** (ruling 289, "Card-sized cap").
  `MAX_BRICKS`, 2,047, was a 1 MiB budget constant from Eponym's residency
  experiment read as a limit, while a 256-tile board with relief needs 5,282
  resident bricks at 1920 by 1080. `AtlasLimits`
  (`crates/conatus/modulus/src/limits.rs`) holds plain numbers a host fills
  from `device.limits()`, the limits its device enforces, and from its own
  budget, 8 MiB recommended, so the crate stays GPU-free.
  `BrickMap::with_limits` takes a brick count, rounds it up to whole atlas
  rows, refuses more bricks than the limits allow and any pointer axis past
  the texture edge, and allocates fallibly. The 16 by 16 slot plane stays
  and rows grow: 8 MiB holds 16,383 bricks at wgpu's default 2,048-texel
  edge, and the 256-texel web tier holds 8,191 whatever the budget.
  `AtlasLimits::DEFAULT` is the old cap, so `MAX_BRICKS`, `from_keys` and
  `with_capacity` behave as before; `from_keys` has no limits-aware twin
  yet. Landed in `0ec498f0`.
- **Verified facts behind that shape.** The pointer encoding needed no
  widening: slots are `u32` in the map, `texture_3d<u32>` in the WGSL and
  `R32Uint` in isometer's tracer. The shader reads the slot plane from
  `BrickTraceSpace.atlas_slots`, so a taller atlas needs no shader change.
  The atlas stays under 4 GiB because its texel offsets are 32-bit in this
  crate and in isometer's slot uploads; widening the plane past 16 by 16
  would need that arithmetic widened first.
- To stay under the 600-line ceiling, the crate's tests and its error type
  moved to sibling files (`076d503e`, `ed0ef4aa`).
- **Consumers adopt it when they repin** (ruling 292). isometer's
  `bricks.rs` wraps `with_limits` beside `with_capacity` and re-exports
  `AtlasLimits`, its tracer fills the limits from the device it owns, and
  Eponym's `StableResidency` can drop its copied `16 * 16 * 512` row
  arithmetic.

## Progress (2026-09-27 brick-traversal precision pass)

Three changes to `modulus`'s shared traversal, ruled by Mark in the wing
design record (`repos/isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`):
ruling 336 on 2026-09-26 ("Fix it in mere"), from the third round of
Isometry's board-paging lane (`testing/scene-board-paging/bands.md` on its
`lane-e-paging` branch), and rulings 361 and 362 on 2026-09-27, from this
pass's findings. Ruling 337 holds that lane's merge until this lands. Branch
`dda-precision`.

- **Finding: the walk's crossings drifted.** `brick_dda` measured each
  axis's first crossing from the start point and then added
  `1 / |direction|` per step, in f32. Isometry's board starts its rays about
  1,380 units behind the ground, so the crossings sit near t = 1,700, where
  an f32 ulp is 1.2e-4. There each addition of 1 / 0.612 rounds down by
  about 6e-5, while the y crossings, stepping by exactly 2, do not. A ray
  that passes a y and an x boundary within 5e-4 to 8e-4 of each other steps
  x first once enough steps have passed. Headroom lengthens the walk, so one
  spare layer moved 330 texels and two moved 660, and against an exact walk
  neither picture was right.
- **The instrument, landed before any fix** (`5c8379d4`,
  `crates/conatus/modulus/src/traversal_tests.rs` and its
  `traversal_tests/` files). `mirror.rs` was the shader in f32 on the CPU,
  operation for operation; since `5e46956a` that walk is the public
  `BrickMap::trace` (ruling 361, below), and the old walk is frozen in
  `traversal_tests/accumulated.rs`. `exact.rs` walks the same f32 ray in f64 with
  every crossing taken from the eye, keeping the shader's rules (the 1e-4
  start offset, the 1e-6 parallel threshold, the x-then-y-then-z tie order
  and the 1,024-cell budget), and records each ray's closest call to a tie
  in f32 ulps. `scenes.rs` rebuilds the probe's 890 by 752 frame bit for bit
  from its camera, over the probe's pointer box at zero to two spare
  layers, and seeds a spread of 48 boxes and 98,304 rays from the origin to
  240,000 voxels out: eyes near, far and inside, grazing rays, rays nearly
  parallel to an axis, and orthographic bundles. A landing unlike the exact
  walk's is a *fault* when the ray never came within 3 ulps of a tie, and a
  *tie* otherwise. On the old shader it reproduces the probe exactly: texel
  (445, 153) lands one voxel over in x at headroom 1, and 330 texels move at
  one spare layer and 660 at two.
- **Candidates measured**, each on the CPU and through its own WGSL on the
  GPU, over the whole board frame at headroom 0, 1 and 2 and the whole
  spread. Cost is a board frame at headroom 1 against the old shader, the
  range over an RTX 4060 and a Radeon 780M through wgpu on Vulkan and DX12.
  Between runs these timings wander by about ten percent.

  | Crossings | Board faults, headroom 0 / 1 / 2 | Texels moved by 1 / 2 layers | Spread faults | Cost |
  |---|---|---|---|---|
  | Accumulated (the old shader) | 2 / 332 / 662 | 330 / 660 | 47 | — |
  | First crossing plus n steps | 0 / 0 / 0 | 21 / 40 | 44 | −2 to +8% |
  | **Taken from the eye: `(boundary − eye) / direction`** | **0 / 0 / 0** | **0 / 0** | **0** | **+1.5 to +12%** |
  | From the eye, times a reciprocal | 0 / 0 / 0 | 0 / 0 | 0 | +1 to +18% |
  | Re-based at the box entry, accumulated | 0 / 0 / 0 | 8 / 30 | 44 | −1 to +1% |
  | Re-based at the box entry, from there | 0 / 0 / 0 | 8 / 30 | 44 | +6 to +12% |
  | From the eye, compared cross-multiplied | 0 / 0 / 0 | 0 / 0 | 0 | +9 to +18% |

  Only crossings taken from the eye leave the walk independent of where it
  starts. A crossing anchored to the start point or to the box entry still
  moves texels with headroom, and a re-based origin carries its own
  rounding, a fault source once the box is 24,000 voxels out. The two
  re-based candidates start 1e-4 past a rounded entry point, which resolves
  the start offset near the origin but not far out; the rest keep the old
  start voxel. None fixes the start offset everywhere (the finding below).
  Only the three taken from the eye pass the headroom test, and of those
  the plain division is the simplest and costs least. The re-based,
  accumulated walk costs nothing but still moves texels, so what the fix
  costs is the price of passing that test, not a choice between exact
  candidates.
- **Landed in `a404cd48`: each crossing is taken from the eye.**
  `brick_crossing` in `crates/conatus/modulus/src/brick_dda.wgsl` computes
  `(boundary - eye) / direction` when the walk enters a voxel, and the
  `delta` vector and `brick_initial_crossing` are gone. Over the whole frame
  and the whole spread, the new walk has no faults at any headroom and no
  texel moves with headroom. Every ray it lands differently from the exact
  walk is within 2 ulps of a tie, all but 2 of them within 1: 46 on the
  board, 7,640 in the spread. On all four GPU configurations the shipped
  WGSL lands every board texel where its CPU mirror does, at every
  headroom. It costs 1.5 to 12 percent more a board frame, 0.02 to 0.08 ms
  for 669,280 rays. The old walk stays in the tests as their positive
  control: it asserts the probe's 330 and 660 in the same run that asserts
  0 and 0 for the shader's walk. The GPU arm runs outside the repository,
  since `modulus` keeps no GPU dependency; its source and logs are kept as
  receipts (below). `BRICK_DDA_WGSL`'s rustdoc states the crossing rule.
- **Finding: the start offset is below f32's resolution past t of about a
  thousand.** `start_t = enter + 0.0001` lands within an ulp of `enter`
  once the entry is past about a thousand units, and on `enter` itself past
  2,048. The walk then starts on the far side of its entry face and steps
  in through it. It reaches the same voxels, but a ray that hits the voxel
  just inside the face reports that face instead of the default
  `(0, 1, 0)`. The GPU fuses `eye + direction * start_t` and so picks the
  side differently from a CPU mirror: 3,754 (NVIDIA) and 4,093 (AMD) of the
  spread's 98,304 rays, nearly all of them first-voxel hits and every one a
  tie against the exact walk, and none on the board. A trial variant that
  clamps the start voxel into the pointer box, and treats an interval
  shorter than the offset as empty (as isometer's CPU mirror already does),
  took those to 10 on every adapter, each a tie, with its cost inside the
  timings' noise. It went beyond ruling 336 and settles which face a
  first-voxel hit reports, so it went to Mark, who ruled it in (362,
  below).
- **Ruling 361, "Call modulus's walk"** (2026-09-27; landed in
  `5e46956a`). `modulus` makes its exact CPU mirror of the shader public,
  and isometer wraps it instead of keeping its own copy.
  `BrickMap::trace(eye, direction, far) -> BrickTrace`
  (`crates/conatus/modulus/src/trace.rs`) walks the map's pointer volume
  and atlas as `brick_dda` walks their textures, in f32 and in the
  shader's order. `BrickTrace` is `Hit(BrickHit)`, `Clear`, or `Exhausted`
  when the 1,024-cell budget runs out first, which the shader draws as a
  miss but a pick should not take for clear air. `BrickHit` is the
  shader's `BrickHit` (`material`, `t`, `normal`) plus the `voxel`. The ray
  is taken as the shader takes it, unnormalised and unchecked, so
  normalisation, error mapping (`Exhausted` to isometer's
  `TraversalLimit`) and the hit point stay with the product. The move
  changed no behaviour: the instrument, now driving the public method,
  gave the numbers of `a404cd48`. `crates/conatus/modulus/tests/trace.rs`
  tests the walk through the public surface alone.
- **Ruling 362, "Adopt the clamp"** (2026-09-27; landed in `c3e054d6`).
  `brick_dda` and `BrickMap::trace` clamp the first voxel into the pointer
  volume, together, and treat a path through it shorter than the start
  offset as a miss; without that rule the clamp would turn such a grazing
  ray into a hit. The exact walk takes the same rule and no longer excuses
  a first-voxel face difference as a tie, so the instrument counts one as
  a fault. The board frame is unchanged: no faults, no texel moved by
  headroom, 46 rays within an ulp of a tie. Over the spread the shader's
  walk has no faults and lands differently from the exact walk on 146 rays
  instead of 7,640, every one within an ulp of a tie, while the frozen
  accumulated walk, which never had the clamp, shows 7,280 faults. On all
  four GPU configurations the committed shader and `BrickMap::trace` part
  on none of the board's 2,007,840 rays over the three headrooms and on 10
  of the spread's 98,304, each within 0.73 ulps of a tie, where before the
  clamp they parted on 3,754 to 4,093; the harness asserts that every
  parting is a tie. Against origin/main's shader the recorded runs cost
  +9.1% (RTX 4060, Vulkan), +13.5% (780M, Vulkan), −1.1% (RTX 4060, DX12)
  and +14.2% (780M, DX12) a board frame, at most 0.26 ms for 669,280 rays.
  Repeated runs of the same code on the 780M gave +0.5% to +21.5%, so read
  that as a few percent to about fourteen, of which the clamp adds nothing
  measurable.
- **Cross-multiplied comparisons: not adopted**, as recommended. None of
  the four GPU configurations divided differently from the CPU walk.
- **Consumers.** isometer's tracer prepends `BRICK_DDA_WGSL`, so its
  pictures take both changes at the repin, and every product's pictures
  move toward the exact walk; their picture receipts are re-recorded then
  (ruling 336). At the same repin isometer's `trace_ray` in `bricks/ray.rs`
  drops its copy of the walk for `BrickMap::trace`, keeping only its
  wrapping (ruling 361). Within this repository no crate consumes the
  shader, and no existing test's expected values changed.
- **Receipts, kept out of tree** in
  `Code/testing/mere/receipts/2026-09-27/dda-precision/`. The five survey
  logs came from the harness in `survey/`, whose `#[path]` includes name
  this worktree; `survey/included/` holds those files as they stood when it
  ran (`mirror.rs` and `scenes.rs` at `5c8379d4`, `exact.rs` at
  `a404cd48`). `survey2.log` predates its adapter listing and interleaved
  timing, and `survey3.log` is adapter 0. The clamp logs came from the
  harness in `clamp/` (its includes in `clamp/included/`, at `c3e054d6`),
  and each opens with its commit, command and compiler, as does the
  release run of the ignored `receipt` test. Adapters 0 to 3 are the RTX
  4060 on Vulkan, the 780M on Vulkan, the RTX 4060 on DX12 and the 780M on
  DX12. `SHA256SUMS` covers every file there, both harnesses' sources
  included.

  | File | Bytes | SHA-256 |
  |---|---|---|
  | `survey2.log` | 10,311 | `a2aded3b94c102740a1c6307247e24d5c478ac0408746165c723a0d95fcf1c57` |
  | `survey3.log` | 11,294 | `55481766e56bff75630176ba55234725b11b2c24bd8ead0dd16ee8eedd62d89f` |
  | `survey-adapter1.log` | 11,312 | `272dc96e03ba5502d79d5e6b6043605d9fc14477a93f916798639e7831394215` |
  | `survey-adapter2.log` | 11,291 | `d838b2380bf9ee6691124c9ffdd1f56cd5611afb106d1a2e7abd797a2eb5c211` |
  | `survey-adapter3.log` | 11,282 | `e4e5a864c616da6b5b02af468c367ce2cdc53fdf59a90cfbbaa0a19eba4c0d4f` |
  | `clamp-adapter0.log` | 1,242 | `e5bdeed49464a81bb03c183772112034275a2e41f8534e470d46ef85480eb209` |
  | `clamp-adapter1.log` | 1,261 | `9a93d66dfbe13ca5b8de2ff85a8da4e11453a173b7bfcd44b404e7174ded22ec` |
  | `clamp-adapter2.log` | 1,240 | `1ff0178adafb5aa96b8f52c1342a08a4204750ece9959fc0e8c6184655ed4773` |
  | `clamp-adapter3.log` | 1,234 | `e87cc1a976398233154d8ee6b15f9e61ab8026d2dd2ed52525cf6015e76d2ed8` |
  | `modulus-receipt-release.log` | 2,353 | `f3e6436ab7ebe20be2eb793ffa8ff785a152b158e1d68989b8200ff4b88771ed` |
  | `SHA256SUMS` | 2,612 | `df10dc7594331a5ac3b9d694dc2c83915af3f1dcb767afc3c608428203be4d3f` |

## Findings and progress (2026-09-28 ownership preparation)

Read-only source verification at Mere `5ce144ff` preceded this documentation
change; the previously recorded implementation and hardware receipts above
were not rerun.

- `crates/conatus/nisus/src/lib.rs:97` defines `VoxelPatch`, with occupancy
  lowering at line 107; `VoxelChunk` starts at line 143 and revision-gated
  `apply_edits` at line 234. This is the existing chunk-mechanics foundation,
  not T2's world chunk map and retained revision log.
- `crates/conatus/conatus/src/engine.rs:166` exposes `bodies_mut`.
  `crates/conatus/conatus/src/world.rs:430` exposes `edit_voxels`, line 468
  `raycast`, line 501 `overlaps`, and line 515 `step`. No public body-binding
  module or query-refresh API is present in the inspected checkout.
- `isometry/mesocosm/crates/mesocosm-runtime/src/tactile.rs:151` still uses
  `step(1e-6)` to refresh queries. Replacing it depends on the separate
  Conatus refresh implementation, verification and product repin; this
  documentation update does not claim that adoption.
- §1 now carries the body-binding shape from the wing's Appendix A; §2
  carries the T2 rulings and migration gate. No binding, refresh, storage,
  or product code changed. The `Engine` source comment and the separate
  runtime-composition acceptance ledger still need their corresponding
  reconciliation; they were outside this preparation lane's file ownership.
