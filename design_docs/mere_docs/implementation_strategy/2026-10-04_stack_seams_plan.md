# Stack seams plan: catalog, shared device, actions, determinism, two words

**Date:** 2026-10-04
**Status (2026-10-04):** plan. Ten rulings in two rounds (S1 to S10); P1 and
P2 for this plan's lane; P3 and S7 done as documents; S3 to S6 carried into the
dynamics grammar plan (G8, G9); S9 done by the identity lane (`b52edea7`).
No code in this plan's own lane yet.

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

## 3. Phases

### 3.1 This plan's lane

- **P1. One arrangement catalog (S1).** A compile step in scenomise, which
  holds the registry and already depends on sceno, takes a scenograph
  definition and a resolved dataset and returns a `sceno::Score`, resolving the
  arrangement id through the registry with parameters from the definition.
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
  all windows) or each window paces itself.
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
- **2026-10-05.** P1's owner settled: this lane, by Mark's answer relayed and
  recorded at `1a2e71db`. S5 and S6 needed no change in the projection grammar
  documents, which already use "arrangement" for positions and "scene" in the
  projection sense.
- **2026-10-05.** S9 done by the identity lane (`b52edea7`); checked here
  against the tree (one `PersonaKey` definition, no `PersonaId` left in mien,
  no outside uses, personae's `PersonaId` unchanged). Its test run (mien's 59
  tests; gemot, moothold and distillery checked) is that lane's report, not
  rerun here.
