# Dynamics grammar

**Status:** 2026-10-28: cut from `2026-10-02_dynamics_grammar_plan.md`, `2026-06-13_scriptable_field_regions_plan.md` and `2026-06-24_meaningful_physics_signals_plan.md`, archived in `archive_docs/2026-10-28/`. Built: declarations and instruments (G1), one channel registry with the cartography disclosures and the physics channels (G2, G2b, G2c), combinators and currencies (G3), the portable `DynamicsSpec` and its binding (G4a, G4b1), arrangement roles (G7), permitted actions (G9), the `state-witness` crate (G8a), rapier 0.36 with enhanced determinism, and the field clock, membership and applied-effect tracing runtime. Open: composition pickers and cheap runners (G4b2), raw terms, realizations (G5), satisfaction reports (G6), the rest of determinism (G8), input record and replay (G10), the visible explanation and acceptance of coupling and field effects, field authoring (P2 to P4, per-field response, general editing), the meaningful-physics slices, and the link model's deferred dynamics. The last recorded pace is F199's pause.

**For:** one specification model of how the graph canvas moves (F1, "terms and targets"): every term declares what it is, composition has declared semantics, choices travel as a `DynamicsSpec`, integration, annealing and one optimizer are realizations gated on class, constraints report whether they were satisfied, and runs repeat from a seed and replay from a record. It also holds the field and coupling runtime that lets fields move bodies and explain themselves, field regions as authored objects, the binding from true signals to physical parameters, and the link model's dynamics deferrals. The grammar was opened by Mark's 2026-10-02 rulings F1 and F2 (P7 of the physics catalog moves here).
Not here: P5 (repulsion on the host's GPU) and P6 (Density), which this plan consumes as rungs and as a law (`2026-10-28_physics_catalog_plan.md`); the Scenograph dynamics slot SE69 and everything built under it, namely the recipe's opaque `dynamics` carrier (F192), the two-arrangements rule (F193), a variant's preset-or-spec choice (F194, F201), the seiche `settle` function (F195), the grid's dynamics axis, occurrence previews and the 60-tick step limit (F200, F202), which the Codex agent builds under SE87 while the physics coordinator reviews (F198) (`2026-10-28_scenograph_editor_plan.md`); the backdrop Reset baseline and interactive scenery (B1, same plan); the forme draft's visual test handoff and the declarative presentation-rule editor (same plan); the projection grammar catalog's section 6 row and the policies it cross-references (F29), and projection-rule precedence (`2026-10-28_projection_grammar_plan.md`); theme and typography roles (`2026-10-28_appearance_plan.md`); link reading, assertions and promotion (`2026-10-28_link_model_plan.md`); the voxel store T2 (`2026-10-28_conatus_engine_plan.md`) and the genet adoption D21 (`2026-10-28_workspace_hygiene_plan.md`), both paused with G4b2 under F199; the storage seam for field attachment and authored attributes (`2026-10-28_graph_semantics_plan.md`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings. The old plan's rulings F1 to F205 stay in the archived copy in long form; only those that bind an open track are carried below.

## Constraints

Shape of the model
- **F1, terms and targets.** One model of energy, dynamics and target terms; arrangements stay `Score`-recorded generators referenced as targets; optimizer realizations are gated on class and metric. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:36`.
- **Scope.** New laws or arrangements, 3D and any change to `sceno::Score` are outside the grammar. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:9-13`.
- **F4, the spec's home.** `DynamicsSpec` lives in seiche with sources resolved host-side and `PhysicsChoice` as the binding; the shelfmark delta section is not built until a citing consumer asks. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:51`.
- **F5 and F96, the common scale.** Weight 1 is a term's force at its declared reference (contact distance 36, one rest length of stretch, one of offset). An entry's weight is a multiplier; a term override is absolute where the term has a reference and a multiplier where it has none, derived and never written. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:55, 251`.
- **F94, F95, F106, F107, F181.** An entry is a catalog preset or a raw seiche term; the spec is a recursive tree; all 20 raw force types take parameter structs, one kind per force type, with graph-derived inputs as channel ids in named slots; a raw slot names only the families its preset slot names (masses `mass.*`, kinds `kind.*`, groups `groups.*` but not `groups.bridges`, depths `depth.*`, pairs `distances.*` and `edges.*`) and radii are refused by slot and id. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:247, 249, 271, 273, 427`.
- **F97, F98, F143, F154.** An unknown kind, term, id or newer version fails the open naming the id and its place; nothing is silently substituted. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:253, 255, 349, 373`.
- **F105, F134, F157.** The spec owns the target (arrangement, anchored pull, default role, group roles keyed by a `groups.*` channel, item roles by UUID); a non-site group is labelled by its smallest member's id; the spec is a recipe, so a host-given partition refuses the save, a dormant law is dropped and a schedule reruns from stage 0. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:269, 331, 379`.
- **F160, F162.** The canvas's public physics surface is `set_dynamics_spec` and `dynamics_spec()` with `PhysicsChoice` as the flat view; the remote board takes `set_stage(&DynamicsSpec)`. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:385, 389`.
- **F101, F156.** One spec-level seed, defaulting to `LAW_SEED`, passed to Kinds and Anneal through `LawSources`. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:261, 377`.
- **F102, F111.** Bars travel in the spec, at the root only, and are evaluated by G6 and the receipts. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:263, 281`.

Realizations and satisfaction
- **F6, F8, F16, F103.** Annealing is a realization of any all-E composition and `anneal.davidson-harel` keeps opening as Springs' terms under annealing; Hub room, Hub pull and Flow's needle are declared as they are, revisited when an optimizer rung exists; a class describes the law and an approximation's fidelity (Barnes-Hut at theta 0.5) joins G5's realizations; until G5 the realization is `Integrate { damping }` and the rung is reserved and refused if set. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:60, 69, 87, 265`.
- **F14, F15.** The class-agreement test asserts Jacobian symmetry for terms with no energy; a velocity-reading term declares N and is measured by the velocity check. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:83, 85`.
- **F19, F26, F46.** Anchored means returns to the arrangement, pinned means stays in it, seeded only starts the motion ("encourage" and "ensure" are retired); a settle is the bodies' rms speed falling under 1 px/s. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:93, 107, 143`.

Determinism and gates
- **Stack seams S4, wing rulings 604 and 609, F116 to F119, F165.** Ordered, declared, cross-platform determinism; every float path a game hands to the sim replays bit for bit on any machine, in conatus too; one labelled digest built in mere and shared (`crates/system/state-witness`, unpublished, exporting `hash_bytes`, Isocosm retiring its copy); rapier's `enhanced-determinism` is on. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:527, 764-805, 291-297, 395`.
- **F138, F139, F166.** A gate reruns only when its inputs changed (the crates whose inputs a merge changed are compiled and tested; headed sets, release receipts and the wasm bundle run only when the change reaches what they measure); a receipt or golden that moves stops the lane and is reported with its figures before any bar changes. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:339, 341, 397`.
- **Lane rules (a reading gathered from current policy, not ruled).** Every negative receipt carries a positive control in the same run; a lane stops at any choice with more than one defensible answer and returns it as a fork. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:530-533`.
- **F199, pace.** Land what is done, then pause: open no lane and prompt nothing until Mark says; peers are answered only where something blocks. T2 and G4b2's design round were queued behind it, and it was still in effect on 2026-10-12. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:465, 1696, 1733`.

Binding order
- **F141, F155, F197.** G4b2 follows G4b1; raw terms, per-term overrides and root weights are a lane of their own after G2c and `derive`'s refusal stays until then; G4b2 comes before it. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:345, 375, 461`.
- **F78, F108, F133.** G4b2 carries the composition pickers, the cheap runners (nested mixes and groupings, overlays inside a node) and the role-group picker; a schedule inside a force returns as its own forks. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:209, 275, 329`.

Fields and runtime
- **F203, F204, F205.** The field clock is accumulated simulation time, Pause freezes it and Reset rewinds it, kept apart from the solver step counter that ages lagged results; spatial eligibility tests each appearance's reference point, boundary included, after the query picks candidates, and camera motion never changes membership; initial matches are reported without entry actions and entry actions begin on later transitions, causes retained. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:473, 482, 493`.
- **F174, F180.** The `edges` channel family (first option `edges.spanning`) holds named subsets of the physics view's edges, carried as `ChannelValues::Edges`; "prospectively", open to revision when typed edges are designed. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:413, 425`.
- **Foreground pin, position anchor, position pin and layout lock stay separate controls;** a lock does not pause dynamics. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:71-77, 147-150`.
- **A painted region or a forme creates no synthetic force field;** an effect edit addresses its own coupling, never the field-wide strength shortcut that sets every coupling. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:170-178, 281-285`.
- **Rhai is not chosen as the universal backend;** Scenograph's scripting comparison and the application execution owners decide. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:66-69`.

Meaningful physics and links
- **The test for every physical behaviour:** it carries a true quantity or affords a real action, never a faked animation; a push is low frequency, per-node updates happen on change, simulations are capped and self-throttle under load. From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:10-17, 132-141`.
- **Physics may suggest graph-truth changes freely but commits them only through confirmation,** one toggle, on by default; collision relations and scriptable tags are each optional. From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:121-130`.
- **Link model L9 and the forces-on-links reading.** Strength defaults to 1 per held assertion until this plan sets it; fields and sympathetic influences act as forces on links, never as extra assertions. From: `archive_docs/2026-10-28/2026-10-15_link_model_plan.md:509-510, 1094-1096`.

## Phases

### G4b2. Composition pickers, cheap runners and the role-group picker
Mixes, groupings and schedules run only through the canvas API; no picker builds them. `derive` refuses nested mixes and groupings, overlays inside a node and any shape the canvas does not run, naming the path. F49's role-group key is a session-only canvas setting with no picker or saved field; the spec's target carries group roles and the picker surfaces them in both web UIs. Lives in `crates/canvas/pictograph` and `ports/graphshell`. The design round is queued and paused under F199.
- Done when: not written in the source. G4's list names what must stay true: the remote board's receipt and the eleven law receipts stay green, and an unknown term kind or channel fails explicitly.
- Needs: G4b1 and G2c (both built)
- Rulings: F78, F105, F108, F133, F134, F141, F197, F199
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:209, 275, 329, 345, 461, 674-704`.

### RT. Raw terms, per-term overrides and root weights
`derive` refuses all three until a lane of their own (F155). The 20 raw force types already have parameter structs with opaque channel-id slots, and G2c made those slots resolvable against the registry; running them is what remains.
- Done when: not written. G2c's done-condition asks only that a raw term's input slot resolves against the registry, which is met, "though running raw terms stays where its own ruling puts it".
- Needs: G4b2 first (F197)
- Rulings: F94, F96, F106, F107, F155, F181
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:375, 461, 642-653, 704`.

### G3r. The within-group stress bar and Density's Bonds overlay
Grouped Charge-between layouts separate meaning clusters, but the within-cluster stress bar is not asserted: as written (Springs alone on the whole graph) every reading fails it, and F72 holds it open until a fixture whose topics have more internal structure exists. Separately, edges may join Density as a Bonds overlay in Density's currency only if its mass-area rank holds at least 0.8; it failed that bar when measured, and Density admits Hub room, Centre and Tide only (F73, F81).
- Done when: the grouped composition keeps edge structure inside each cluster, with a within-cluster stress figure no worse than Springs alone, on a fixture whose topics have internal structure; for Bonds, a formulation that holds rank 0.8.
- Needs: a fixture with internal topic structure; the physics catalog plan's Density bars
- Rulings: F72, F73, F81, F82, F182
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:197, 199, 215, 217, 519, 663-672, 1293`.

### G5. Realizations
Annealing becomes a realization of any composition whose terms are all E under one metric (Metropolis moves over the declared energy, cooling schedule a realization parameter), retiring Anneal's private energy copy (`crates/conatus/seiche/src/laws/anneal.rs`, lines 66 to 91). One optimizer rung is gated the same way and refused with its reason for any composition holding an H, N, K or mixed-metric term. P5's repulsion rungs and Density's tiers are recorded as rungs of their terms; a rung must reproduce its term's law to a stated tolerance. Time-dependent or extent-clipped gradient terms already declare nonautonomous N and expose no timeless global energy.
- Done when: annealing is selectable for every all-E composition and refused with its reason otherwise, with a test per class; `anneal.davidson-harel` reopens from a saved scene and its P2 receipt (energy at most 5) stays green; on the P2 fixture the optimizer rung reaches an energy no higher than integration's (within a stated tolerance) for Springs, Stress and Energy, and each law's signature receipt passes on its result; compositions with Orbit, Sync, Kinds, Flock, Flow or a hub overlay mixed with identity-metric terms are refused with their reasons; F8's figures for making Hub room, Hub pull and Flow's needle gradients are in front of Mark; the eleven law receipts stay green.
- Needs: G1 and G4 (built); the physics catalog plan's P5 and P6 tiers for the rung records
- Rulings: F1, F6, F8, F14, F15, F16, F103
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:718-734, 873, 1748-1751`.

### G6. Satisfaction reports
Constraints report whether they were satisfied, the physics counterpart of `sceno`'s honored and unmet holds. Pins (kinematic bodies) report honored or unmet against a tolerance, anchors report their residual RMS, contacts keep reporting overlaps, and the figures reach `LayoutStats` and both hosts' snapshots. When a spec travels with a score, an unmet pin maps to `unmet_holds` (a reading, not ruled). The bars carried in the spec (F102) are evaluated here.
- Done when: a pinned node reports honored; a planted unsatisfiable pair of pins (two bodies pinned to one point against contacts) reports unmet, the positive control; the anchor residual falls monotonically as `arrangement_pull` rises across a sweep; the reports reach the web snapshot and turnstone's observe snapshot; the eleven law receipts stay green.
- Needs: G4 (built)
- Rulings: F19, F26, F46, F102, F111
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:736-750`.

### G7t. Hosts onto the roles
G7 is built on the canvas and the practice board. The remote board's slot pull (0.5), swatch relax's pull (0.25) and mer3ly's default (Anchored at 13) follow onto the seeded, anchored and pinned roles as each moves; mer3ly's Anchored and Free mobility follows when mer3ly consumes the board. The canvas's default has already moved from pull 12 to the seeded role (F23).
- Done when: not written; the source says only that each follows "as each moves onto the roles", and G7's own done-conditions are met.
- Needs: each host's own change; mer3ly consuming the board
- Rulings: F18, F19, F22, F23, F26
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:101, 822-832`.

### G8. Determinism, the remainder
The labelled digest crate (G8a) and rapier's `enhanced-determinism` (with the rapier 0.36 move) are built. The probe found three sums over the per-process-randomized body map (`barnes_hut.rs:114`, `forces.rs:235`, `laws/hold.rs:28`); the speed lane's key-order ruling ordered NodeExclusion and Barnes-Hut only, so what remains is for the probe and the lint to show. The rest of the list below is open.
- Done when: the `seiche-repeat` probe prints one fingerprint across five processes (today's five different ones are the control); each `Term` declares deterministic, seeded or nondeterministic and an undeclared one does not compile; the probe's fingerprint matches on Windows and on macOS or Linux; the cost of `libm` in seiche's laws is measured at the probe's sizes, and checkpoint C3 applies: if it costs more than the run-to-run spread at 5,000 bodies the figure returns to Mark before it is kept; a per-tick trace with labelled slices reports the first diverging tick and slice; velocity is in the hash; one owned random generator per world replaces seiche's xorshift32 and SplitMix64; a lint rejects bare transcendentals and hash-ordered loops; and in conatus (wing ruling 604) glam runs with `libm` and `scalar-math`, the lint holds, and a digest is diffed across Windows, macOS and Linux.
- Needs: the Windows and macOS or Linux machines for the cross-OS fingerprint
- Rulings: stack seams S4, wing 604 and 609, F101, F116 to F119, F156, F165
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:527, 752-804, 1366-1367`; stack seams checkpoint C3 `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:988-989`.

### G10. Input record and replay
Balaur review fork E ("That, plus replay as a track"): seiche's `LayoutSnapshot` holds positions only, so a run needs an input record and a restorable checkpoint.
- Done when: a run's inputs (pointer drags, pins, law and dial changes, seeds) record to a JSON Lines file with a header, one line per tick carrying that tick's digest, and a trailer whose absence means the run crashed; replaying it on another machine reproduces every per-tick digest; a restorable checkpoint holds positions, velocities, generator state and each term's state, and restoring it mid-run continues to the same digests; positive control: a planted extra input, or a perturbed checkpoint, diverges at the tick where it was planted and the G8 trace names it.
- Needs: G8's trace, digest and owned generator
- Rulings: balaur fork E; its order against G3 to G6 is Mark's
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:806-816`; `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:62`.

### C1. Visible explanation of contributions
The engine side is report-only: a point evaluation reports the proposed write, and an opt-in trace records executed calls with exact bindings, latest tick only, with a caller-supplied row cap that discloses what it omitted. Missing: a host presentation of simultaneous forces (vectors, shapes or relationship treatments, candidate forms only), an actual-contribution inspector, composition across fields and its inspector, and a seam binding the evaluated term or spec path to the source assertion or `FieldId` and `CouplingId` and the affected body or appearance (the endpoint-pair spring input alone does not supply it), preserving grouping disclosure where one visible relation cell covers several assertions. A projection explains a term using the primitive identities it actually reads; relationship membership alone installs no force. The edge and resource-association terminology needs graph-semantics reconciliation before it changes any channel or endpoint contract.
- Done when (candidate proof, not opened): inspect an item with two active contributions and an arrangement constraint; distinguish the contributions, the resultant and constraint satisfaction, with a control that disables one contribution; derive the explanation from the terms actually evaluated.
- Needs: `2026-10-28_graph_semantics_plan.md` for the terminology; `2026-10-28_scenograph_editor_plan.md` for the primitive and dynamics map
- Rulings: F203
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:945-979, 1642-1651, 1694-1696`.

### C2. Field membership and entry-action policy
F203 to F205 are built. No policy exists for attached extents (they need an owner resolver and are reported unsupported), footprint intersection and containment as later modes, jitter, debounce, hysteresis, re-entry and swept-crossing inference, applet execution, conflicting state writers, or source scripts and a domain receiver. Observation is at tick sample points.
- Done when: not written beyond the source's statement that entry-action execution, visible authoring and an application-specific script receiver are separate owner acceptance cases.
- Needs: the owner of attached extents; `2026-10-28_graph_semantics_plan.md` for attachment
- Rulings: F203, F204, F205, F199
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:482-503, 1713-1725, 1762, 1782-1804`; `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:262-293`.

### C3. Consumer adoption and coordinator review
Knot's relationship consumer is qualified within its own bounds; panel cropping and offscreen bodies are known visual defects and no full visual acceptance is claimed, and the canvas receipt carries one reproduced baseline golden failure (topic-radial), so there is no all-green stack claim. Firefox verification is deferred while the Windows helper lacks URL-policy support. Coordinator-owned Mere/Knot and Djinn/Knot-site adoption pins are unchanged, and the physics coordinator's independent review remains open. Field provenance, named channels and groups, nonflat scenes, ambient inference and unsupported body forms stay outside the binding.
- Done when: not written in the source: visible host explanation, external consumer adoption and coordinator review are recorded as separate acceptance work.
- Needs: C1; `2026-10-28_scenograph_editor_plan.md` for the slot's own review
- Rulings: F198, F199
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:1548-1550, 1574-1576, 1596-1616`.

### C4. Downstream repins
Consumers change at their next repin: Knot's `load_wgpu` call in `knot-editor/crates/knot-editor/src/embedding.rs` must pass a device (F52, Knot first and briefly red); Turnstone's `project_spiral_score` call (`src/remote_projection.rs`) and the `project_canvas_strategy_with_score` calls in Turnstone and cleromancy change with F87 and F132; Turnstone's by-id actions and woodshed's `board.set_choice` move onto `set_dynamics_spec` (F160, F162).
- Done when: each consumer builds against the changed signature at its repin; the source states nothing more.
- Needs: each consumer's own repin
- Rulings: F52, F87, F132, F160, F162
- From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:155, 231, 327, 385-389, 1022, 1061`.

### FR1. Hosts for field placement, strength and response
The library places, moves, resizes and re-aims a force well (P0, P1) and sets runtime damping and per-field strength, but no host calls `add_field_at` and none drives `set_field_strength` (meerkat's roster was its control). Per-field response (gather, repel, wall, dampen) is absent; field removal dropping its force travels with it, on the physics settings surface.
- Done when: P0, you can place a visible field at the cursor, see it and drag it; P1, nodes in the region drift per its force rule and a force tick reflects it (both met in the library, with no host caller).
- Needs: a host with a placement gesture and a control for strength
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:5-13, 614-624, 712-725`.

### FR2. Edge visibility, layout and the rule surface (P2 to P4)
P2 scopes an `EdgeProjectionSpec` to a region's nodes (`show_edges`, `hide_edges`), P3 applies an arrangement to the contained nodes (`arrange(strategy)`), and P4 evaluates one rhai script over a `FieldContext` snapshot to set all three characteristics. No `FieldContext`, `show_edges` or `arrange` binding exists, and the current scalar and vector AST implements neither region events nor applet execution. The graph behaviors plan placed three projection-tier ideas here: nearby-nodes refresh (a spatial-region trigger), a staleness badge (time and a clock) and rollup counts on a container (delta, projection only).
- Done when: P2, a region can reveal or hide relation families for the nodes it contains; P3, a region re-lays-out its contents; P4, one region script sets all three characteristics over its contents.
- Needs: FR1; C2; Scenograph's scripting comparison for the backend
- Rulings: F203, F204, F205
- From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:66-69, 577-581, 598-602, 614-653`; `archive_docs/2026-10-28/2026-08-13_graph_behaviors_plan.md:446-465`.

### FR3. General field authoring
Edit, Preview, Apply and Discard for a general field's authored values, in the sections Members, Extent, Placement, Influence, Projection, Actions and Appearance. Preview reports added and removed members and affected placements without invoking applets or mutating source; Apply validates the whole draft before any durable write and addresses the existing Field and Coupling or workbench owner; a field and its changed couplings need one coherent owner edit that graph deltas do not by themselves provide. Exact-coupling strength editing (draft, acknowledged host transaction, saved undo) is implemented; nothing else here is. The proposed membership model keeps eligible subjects, occurrence-local spatial tests and influence weight separate, to be mapped onto the Scope, selector and arrangement owners rather than serialized as a new selector. The design pass was assigned to the projection grammar agent; visible dynamics explanation and script or motion authoring stay with the dynamics lane.
- Done when: two overlapping fields are inspected without hash-order-dependent edits; one named coupling is tuned without changing another; matched and unsupported effects are shown; field identity and authored values save and reopen through the owning host; scoped apply, discard and undo lose no content; the agreed forme draft is qualified against the existing bridge; runtime tests cover repeated appearances, conflicting placement, an empty or limited query result and a failed apply. Entry-action execution is a separate dynamics-owned case (C2).
- Needs: FR1; the conflicting-placement and attachment forks below; `2026-10-28_graph_semantics_plan.md`; `2026-10-28_scenograph_editor_plan.md` for the primitive editor draft
- Rulings: F204, F205 (forks 2 and 4 are unruled)
- From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:43-55, 152-293`.

### FR4. Field inspection residue
Large-session candidate-copy and log-list costs, and the full-list and AST materialization, need checking before adoption on large graphs. The startup builder passes couplings to the force bridge without the active-owner filter, so retired-field force behaviour needs runtime-owner review before it is called qualified. Field-wide strength APIs still affect all couplings and cannot serve one row. Inspector spacing and focus and scroll need design and visual qualification. The host-test handoff's review list stands: keyboard activation and focus, cancel and drag-outside release, narrow and docked layouts, long identifiers and ASTs, source save failure and busy state.
- Done when: not written beyond the handoff's checks.
- Needs: a host with sufficient build memory for the native cases
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:332-333, 452-458, 494-495, 502-529`.

### M1. System pulse and per-node truth (slices 1 and 2)
`AmbientMetrics` and `AmbientSim::set_metrics` do not exist; the first slice feeds one sim (n-body body count as active operations) end to end, and the second maps per-node content state to node heat (Loading pulses, Failed warns, Ready calm) through the material and size seam. Both were specified on meerkat's sources, which are gone; no live equivalent exists for the observability buffer or the steward rows.
- Done when: slice 1, one sim reads a real metrics feed end to end at low frequency; slice 2, a node's content state shows as heat through the existing material seam.
- Needs: a host that exposes fetch, sync and operations metrics, or re-grounding on Turnstone's `ContentStates` and stickleback's `SyncStatus` (open question)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:46-51, 111-112, 143-150, 164-166`.

### M2. The mapping layer, presets and the graph signals (slices 3 and 4)
A configurable signal-to-parameter table with named presets (a `NodeMaterial`, size and behaviour bundle), the user picking which quantity drives which parameter, and conditional presets (a query, tag or state adopts a preset). Slice 4 binds the graph signals already produced in pictograph (importance as weight, affinity as edge tension, clusters as forces) through that layer; its wait is over. Degree stands in until bound.
- Done when: not written beyond the slice text: a configurable table and presets exist, conditional presets work, and importance, affinity and clusters reach physical parameters through them.
- Needs: M1's seams; the graph signals producer in `crates/canvas/pictograph/src/signals/`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:72-78, 116-120, 150-162`.

### M3. Collision relations and scriptable tags (slice 5)
A collision edge grows slightly stronger with each collision and fades with time, and a gesture substantiates it into a real relation. Tags bestow physical traits through a script lane (rhai today), with configurable autotagging. Both are optional and sit behind the confirmation default.
- Done when: not written; each is optional.
- Needs: M2; the confirmation default
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:121-130, 158`.

### PS. Physics scenes, tangibility and the tactile profile
Carried from the archived plan tails, none gating: a Mere or Turnstone host scene picker and tangibility binding, a bounds-drain (reap scene bodies by region, not only age), a fluid emitter (spray PBF particles, not rigid droplets), reuse of numen's `CouplingForce` (named aether then) for richer fields on scene bodies, re-applying tangibility on node add, tactile T4 piles (proximity clusters as a derived query over the contact graph, open until a host wants it) and T5, the mere profile (a mere's physics profile as data, wired into a real canvas).
- Done when: not written; T4 is "open until a host wants it".
- Needs: a host that wants each
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:902-908, 1310-1315`.

### LD. Link dynamics
The link model defers to this plan: a weighted strength policy, one aggregated spring per link, measured per-assertion attribution of force, endpoint feedback, and sympathetic resonance. Until the policy is set, strength is 1 per held assertion and current unit source-row contributions are explicit. Fields and sympathetic influences act as forces on links, not as additional relationship assertions. The integrated explanation of pair dynamics, the counted endpoint topology and the independent responsive-path evaluator are qualified in the link model and are what this phase builds on.
- Done when: not stated in the source; the link model's design owns the rulings and this plan sets the policy.
- Needs: the applied-effect trace and explanation seam (C1); `2026-10-28_link_model_plan.md`
- Rulings: link model L9; F5 and F96 for the common scale
- From: `archive_docs/2026-10-28/2026-10-15_link_model_plan.md:44, 477-479, 509-510, 1094-1096`.

## Open questions

- Which optimizer for G5: L-BFGS over the declared energy (OpenMM's minimizer, Penrose) or stress majorization (Graphviz's default for stress). From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:873`.
- Where the effectiveness record lives. The brief argues for a table versioned beside the spec; its first rows are seeded in the archived Findings. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:874, 1003`.
- In what order G5, G6, G8 and G10 run; tracks that could run side by side go back to Mark, and G10's order against G3 to G6 is his. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:537, 816`.
- Whether F199's pause still stands; nothing in the sources records it lifted. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:465, 1733`.
- Typed nodes, edges and fields and which effects interact with which (edges by material, so one interaction cannot cut an edge but can banish it); a world rather than a scene. Open, not scoped; the `edges` family is its first room. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:413, 930`.
- Whether seiche takes `state-witness` as a dependency (then it needs a family-name round and a publish), whether `mere-framing` 0.0.2 was published (F140 ordered it once the fix landed on main; publication is not recorded), and whether mere's roughly ten hand-written FNV-1a copies (mesh, esp, gaz, pictograph, cambium, linked-data and others) move to the witness. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:295, 297, 343`.
- Whether seiche and `scenomise::relax` converge. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:794`.
- Where T5, the mere profile, lives: folded into G4's spec, folded into the physics catalog's named profiles, or kept separate (the tactile plan's own question). From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1311-1315`.
- Whether G2's Meaning closes three archived tails: a BERT `semantic-embeddings` provider upgrade, an off-thread embedding actor for raw-body text, and a caller for pictograph's `set_content_affinity` (the tails say it has none). G2 built Meaning on a pinned e5-base-v2 model through ESP on the host's device, off the frame, keyed to titles or URLs (tags and body left out until a consumer needs them, F33), with an opt-in Meaning-to-Affinity path that is the first non-test caller (F36). Which of the three remain, body text most likely. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1009-1012`; `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:121, 127, 163`.
- Where meaningful-physics slices 1 and 2 get their inputs: Turnstone's `ContentStates` and stickleback's `SyncStatus`, or a host that exposes fetch, sync and operations metrics. From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:164-166`.
- Who owns the scene-wide arrangement choice, also called the arrangement axis; the node-representation plan that held it was superseded on 2026-06-23 and the graph signals layer plan it went to was archived on 2026-08-20. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:30-32`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:596-597`.
- The field plan's open decisions: where the rule script is edited (an inspector pane is likely), selector semantics (F204 answers the spatial part), the recompute trigger, and the persistence home for the script. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:637-653`.
- Field forks still unruled: conflicting exact placements (refuse the incompatible edit is recommended; the alternatives are one designated owner or an explicit priority), and how a forme representation, a Numen force field and arbitrary authored attributes associate before persistence changes. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:270-271, 281-285`.
- Field research left open: spatial inclusion against declared membership, overlap and nesting composition without a second workbench layout authority, entry and exit under movement and boundary jitter, whether a layout lock affects local dynamics, the feed-overflow example (an action must say whether it changes appearance, attention, residency, keeping or source truth), and how fields are represented beside nodes and links (address, owned attributes, membership, behaviour, scene representation). From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:71-95, 109-114, 126-137`.
- Contradiction: the field inspection handoff says neither `p4_tree_field_inspection*.scn` scenario has run, while the Chrome continuation records both tracked pages passing on Chrome 154 with IndexedDB reopen. Which holds, and what remains to run. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:487-500, 518`.

## Findings

- 2026-10-02 (G1): Barnes-Hut at theta 0.5 is not reciprocal to the instruments' tolerance; at theta 0 the same term passes everything. Position-only descent tests cannot see velocity-reading terms (Boids' alignment and cruise, Orbit's counter-damping), which `velocity_read` catches. Both shape G5's class gating. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:996, 998`.
- 2026-10-02 (G3 correction): giving every group member its group's full centroid force is class Em (member forces are n_g times the true gradient) and fails reciprocity on groups of unequal size; F9's weight share is the true gradient. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:986`.
- 2026-10-03 (G7): the arrangement pull was removed (`270172db`) and F18 to F30's roles replace it; F47 keeps `arrangement_pull` in `SavedSceneV1` only as the anchored stiffness, with a serde default. Retired, not open. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:1049-1054`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:666-668`.
- 2026-10-12 (coupling): a generic force or strength explanation would misdescribe velocity assignment, multiplicative damping and positional advection; damping ignores coupling strength; the wall is a depth-scaled force, not exact positional exclusion; a point evaluation is not an applied-effect trace, an aggregate force or a validator; the trace's pre-integration observations must not be shown as one coupling's isolated contribution. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:1625-1630, 1642-1651, 1674-1678`.
- 2026-10-12 (field runtime): `reset_field_runtime` rewinds time and bumps the clock epoch; ordinary seeding and binding rebuilds keep time; the clock is local runtime state, never graph truth, sync timestamps or a persisted scene schema. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:1735-1745`.
- 2026-10-11 (Knot consumer): playback advances one fixed tick per eligible frame, not wall-clock simulation; compiled card footprints sit in a fixed physics plane and the display is clamped to it, a presentation choice and not a physical wall. From: `archive_docs/2026-10-28/2026-10-02_dynamics_grammar_plan.md:1596-1603`.
- 2026-10-10 (field substrate): field picking takes the last matching field in hash-map iteration and paint order, an authoring ambiguity and not a precedence; the field-wide strength shortcut reads the first matching coupling but sets every coupling (`crates/graph/graph-kernel/src/graph/field_ops.rs`); the canvas converter takes selector matches independently of extent, which F204's runtime now filters. From: `archive_docs/2026-10-28/2026-06-13_scriptable_field_regions_plan.md:165-178`.
- 2026-10-06 (signals): meerkat's content, sync, observability and steward sources left on 2026-07-18; live equivalents exist for two (Turnstone's `ContentStates`, stickleback's `SyncStatus`); importance, community and affinity are produced in pictograph's `signals` and the affinity spring is built from them. From: `archive_docs/2026-10-28/2026-06-24_meaningful_physics_signals_plan.md:46-51, 72-78`.

## Progress

- 2026-10-28: cut from `2026-10-02_dynamics_grammar_plan.md`, `2026-06-13_scriptable_field_regions_plan.md` and `2026-06-24_meaningful_physics_signals_plan.md`; nothing new landed.
