# Dynamics grammar brief

**Date:** 2026-10-02
**Status (2026-10-02):** research brief, complete; the forks in §11 wait for Mark. Docs only, no code changed.
*Annotated 2026-10-02:* Mark ruled all ten forks the same day: F1 "Terms and targets", F2 "P7 moves into the grammar plan", F3 "Declarations and instruments first", F4 "Portable shape now, in seiche", F5 "Reference-configuration normalization", F6 "Annealing becomes a realization", F7 "Keep (1, −1), relabel ForceAtlas2", F8 "Declare them as they are", F9 "Weight share", F10 "Measure first". They are recorded in the physics catalog plan, §3 P7, and carried in the [dynamics grammar plan](../implementation_strategy/2026-10-02_dynamics_grammar_plan.md), which also corrects §7.2's and F9's description of the full-force spread (its Findings).
**Ruling carried:** the [physics catalog plan](../implementation_strategy/2026-09-02_physics_catalog_plan.md), §3 P7, "Held for a dynamics grammar (2026-10-02)". Mark said: "in the way we've managed to describe projection grammar, we should probably think about an overarching model of physics; combinatorial, algorithmically diverse... more thoughts?" He ruled "Research brief first, P7 held" (a prior-art shelf with one-line transfers, and every current law, overlay and slot decomposed, before P7 is rewritten from it); "Its own doc beside projection grammar" for the grammar's later plan; and, on treating arrangements and laws as two realizations of one objective model, "Yes, as a hypothesis the brief tests". The same plan's 2026-10-02 rulings on currencies, the composition tier, meaning as a source, embeddings and snapshots are inputs here, not reopened.
**Audit base:** mere `a059b838` (main). Two unmerged branches were read with `git show`: `density-cpu` at `1bf47cf9` (Density, P6a) and `gpu-repulsion` at `03cb4207` (P5a–b). mer3ly at `7e455f9`. Web sources fetched 2026-10-02 (§12).
**Related:** [projection grammar catalog](2026-08-15_projection_grammar_catalog.md) (the grammar this one parallels), [projection grammar adoption plan](../implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md) (its Findings transfer table and Track A), [projection scenes and the graph-native platform](../../2026-08-23_projection_scenes_and_graph_native_platform.md), [the cartography–gyre layout seam](../technical_architecture/2026-05-29_cartography_aether_layout_seam.md) (arrangements compute, physics simulates), [mer3ly stack consumer survey](2026-08-16_mer3ly_stack_consumer_survey.md), [shelfmark format note](../technical_architecture/2026-08-16_shelfmark_format_note.md).

*Reading, not ruled* marks this brief's own inference. Every other claim is a quotation, a file and line, or a cited source.

**Path key.** Short paths in the tables expand as follows. `seiche/` is `crates/conatus/seiche/src/`; `pictograph/` is `crates/canvas/pictograph/src/canvas/`; `cartography/` is `crates/canvas/cartography/src/`; `sceno/` is `crates/cambium/scenes/sceno/src/`; `numen/` is `crates/conatus/numen/src/`. The main files are `crates/conatus/seiche/src/lib.rs`, `crates/conatus/seiche/src/forces.rs`, `crates/canvas/pictograph/src/canvas/physics_catalog.rs`, `crates/canvas/pictograph/src/canvas/cartography_scene.rs` and `crates/cambium/scenes/sceno/src/score.rs`. Files that exist only on a branch are cited in git's `rev:path` form, for example `1bf47cf9:crates/conatus/seiche/src/laws/density.rs`.

## 0. The result on one page

- **Decomposition.** Every current law, overlay and slot decomposes into *interaction topology × kernel × state moved*, with a currency. Seven places resist a clean split (§5.6). Sync carries two states in one force. Orbit bundles an initial condition, a kernel and an anti-damping drive. Flow bundles two kernels. Boids reads velocity. Anneal, Density and Hold are realizations rather than kernels. Charge's id names its rung. Damping and contacts live in the integrator, not the force list.
- **The hypothesis holds for one class and fails for two** (§6). "Arrangements and laws are two realizations of one objective model" holds for the *energy* class. That is 6 of the 12 laws (Springs, Charge, Stress, Energy, Anneal as an optimizer, Density in a Wasserstein metric), 5 of the 8 overlays, the anchor and affinity slots, and 1 of the 8 listed arrangements (Spectral, a closed-form minimizer). It fails for the *dynamics* class, where an objective is either missing (Kinds, Flock, Flow's needle) or has a minimizer that is not the picture (Orbit's minimizer is collapse; Sync's is total synchrony). It also fails for the *generator* arrangements (Spiral, Grid, Penrose, Fractal), whose objective is trivial: they assign items to generated slots in order.
- **What survives** (*Reading, not ruled*): a weaker and more useful claim. Arrangements and laws are realizations of one **specification** model whose units are **terms of three kinds**: energy terms, dynamics terms and target terms. Arrangements enter as target generators, and `AnchorSpring` already shows how ("an arrangement expressed as a *field* rather than an override", `seiche/anchor_force.rs:7`). Optimizer realizations are legal only for compositions made entirely of energy terms under one mass metric.
- **Findings** (§5.7) that hold whatever Mark rules on the forks:
  - `energy.linlog` runs ForceAtlas2's (1, −1) force model, not LinLog's (0, −1).
  - Anneal's energy has the same functional form as Springs' energy.
  - The two hub overlays are non-reciprocal.
  - Flow's needle is not a gradient as written.
  - Kinds is non-conservative because its kind matrix is asymmetric, and only for that reason.
  - `never_rests` has excluded Kinds since P1, against the plan's own finding.
  - Group pull reads site only.
  - mer3ly's citation no longer carries physics.
- **Forks** (§11), recommendation first in each: F1 the model's shape; F2 where P7 lives, with a draft rewrite; F3 the grammar plan's first tracks; F4 where the spec lives; F5 the common strength scale; F6 Anneal's place; F7 Energy's kernel; F8 terms that are non-conservative by accident; F9 the grouped spread rule; F10 whether Kinds keeps ticking.

## 1. The question

The projection grammar did one thing that made scenes tractable. It stopped treating a picture as a type and factored it into layers (authority → reading → encoding → arrangement → scene → realization; catalog lines 43–57), with an explicit promotion discipline. Mark's question is whether physics deserves the same treatment. Physics today is a catalog of eleven laws plus Density on a branch, eight overlays, three slots and three source pickers. It risks becoming a list of special cases that compose by accident: forces sum through rapier's `add_force` with no per-law weights (plan, "Assessed and ruled" findings).

The orchestrating session's working sketch is tested here rather than assumed:

1. **Terms, not laws.** Each law decomposes into interaction topology, kernel and state moved. Verdict: holds, with seven exceptions (§5.6).
2. **Sources as channels with scales**, with shared or independent scale resolution, which is the same question as a common strength scale. Verdict: holds, and extends to the mass metric (§6.5, F5, F8).
3. **Scope as selection** (SetCoLa), and **grouped laws as scope plus coarsening** (FM³). Verdict: holds, with a semantic choice about how a centroid's force is spread (F9).
4. **Combinators**: sum, scope, sequence, level-of-detail condition, constraint. Verdict: holds, with per-class semantics (§7).
5. **Currencies as the type system.** Verdict: necessary but not sufficient. Class (energy or dynamics) and metric matter as much as currency (§7.1).
6. **Potential against non-conservative terms.** Verdict: the central axis, and it needs a third case: conservative, but the meaning lives in the trajectory (§6).
7. **Arrangements and laws as two realizations of objectives.** Verdict: partial (§6.4).
8. **Observables as receipts; effectiveness knowledge beside the grammar.** Verdict: holds, and the receipts already exist (§8).
9. **A portable artifact.** Verdict: holds in shape (§9). Its forcing consumer is weaker than it was in August (§5.7 F-h).

## 2. The parallel with the projection grammar

| Projection grammar move | Where it is recorded | Dynamics analogue | Where it sits today |
|---|---|---|---|
| Factored layers rather than monolithic chart types | catalog lines 43–57; GoTree caution, line 430 | term = topology × kernel × state; sources; combinators; realization rungs | fused inside each `Force` (`seiche/lib.rs:258`) |
| Per-item disclosures: axis, embedding, weight | `cartography/adapters/score.rs:33`; Score v4 | sources: kind, mass, depth, pairs, groups | `LawSources`, `pictograph/physics_catalog.rs:599`; affinity pairs separately |
| `ensure` / `encourage` (Penrose); pin against anchored home | catalog line 434; `sceno/score.rs:85` (`Hold::{Anchored, Pinned}`) | constraint terms with a satisfaction class | pins are kinematic bodies (`seiche/lib.rs:829`); anchors are `AnchorSpring`; neither reports satisfaction |
| Level-of-detail rungs as declarative conditions (Gosling) | catalog line 433; adoption A3 | realization rungs selected by measured conditions | P5's "Node count per host, measured" threshold ruling (plan, P5 rulings) |
| Effectiveness knowledge versioned beside the grammar (Draco) | catalog line 427; adoption Findings, last row | which compositions carry which inference, with numbers | scattered across P2 receipts and density probes |
| Set-scoped constraints (SetCoLa) | catalog line 428; adoption A5 | scope as selection; kind matrix as a pair partition | numen `NodeSelector` for couplings only (`numen/coupling.rs`) |
| A portable, versioned record of what was chosen (`Score`, `SCORE_VERSION`) | `sceno/score.rs:35-57` | a dynamics spec | `PhysicsChoice` (`pictograph/physics_board.rs:48`) and `SavedSceneV1`'s physics fields (`ports/graphshell/src/product.rs`, lines 216–243) |
| "Solver proposes, the score records" | adoption lines 194–197 | the dynamics spec is a proposal procedure; realized positions are recorded as placements | A6 closed (adoption lines 264–290): `Placement::Coordinate` and holds carry realized placement |
| A forcing consumer, then a second heterogeneous one | catalog lines 357–400 | the same discipline for promoting a spec | §9, F4 |

The seam doc's finding still frames the split: "cartography strategies *compute* a layout (pure function of input + serializable state); gyre *simulates* one (a stateful actor with collision and interaction)" (seam doc, Finding 1). The hypothesis asks whether *compute* and *simulate* are two realizations of the same thing. §6 answers: sometimes.

## 3. Vocabulary used below

These are plain working words; none is a product name.

- **Topology**: who interacts.
  - *unary*: each body alone, against a target or a field;
  - *edges*;
  - *pair list*: given pairs with weights, such as affinity pairs or a distance table;
  - *all-pairs*: optionally within a cutoff radius;
  - *groups*: members against a centroid;
  - *kind matrix*: pairs partitioned by (kind_i, kind_j), each cell with its own coefficient;
  - *medium*: a grid field the bodies read and write.
- **Kernel**: the functional form of the interaction.
- **State moved**: position (through force → velocity → position under rapier, or written directly), velocity, phase, or a field.
- **Currency** (Mark's ruled terms, plan "Currencies"): *force* adds force and rapier integrates it. *Kinematic* writes velocity or position. *Resident* integrates on the GPU.
- **Class** (*Reading, not ruled*; this brief's taxonomy, derived in §6.1):
  - **E**: a gradient of a scalar energy over positions, with every body at equal inertia. Any optimizer can realize it, and the minimizer carries the inference.
  - **Em**: a gradient under a non-identity *mass metric*. The forces are `−M⁻¹∇U` with `M` read from a mass channel. Alone it descends `U`. Summed with terms under another metric, it is not guaranteed to descend anything.
  - **H**: conservative or gradient, but its minimizer is degenerate, so the inference lives in the trajectory.
  - **N**: non-conservative: non-reciprocal, velocity-driven, or rotational.
  - **K**: writes state. A K element is classed by what it realizes.
  - **T**: a target, meaning a slot set from a generator, held at a satisfaction class.
- **Rung**: one realization of one term (CPU exact, Barnes–Hut, GPU tiled, and so on).
- **Realization**: how a composition is driven to its picture. Closed form, optimizer, integration and annealing are the options.

## 4. Prior-art shelf

Three shelves, as the projection catalog split its own. Each entry gives what the system is and one transfer. Versions are the ones the sources showed on 2026-10-02.

### 4.1 Simulation systems: term grammars that already exist

- **OpenMM** (docs 8.6; <https://docs.openmm.org/latest/userguide/theory/03_custom_forces.html>). Forces are classes added to a `System`. Custom forces are written as *energy* expressions ("the interaction energy of each bond is given by E=f(r)"), and forces follow by differentiation. The custom families are named by topology: `CustomBondForce`, `CustomAngleForce`, `CustomTorsionForce`, `CustomNonbondedForce`, `CustomExternalForce`, `CustomCompoundBondForce`, `CustomCentroidBondForce`, `CustomManyParticleForce`, `CustomGBForce`, `CustomHbondForce`, `CustomCVForce`. **Transfer:** a term is *(topology family, energy expression, per-item parameters)*. Authoring the energy and deriving the force makes potential-ness true by construction. numen's field AST (`numen/field_ast.rs:43`) is already this for unary terms.
- **OpenMM's standard forces** (AMBER and CHARMM functional forms; <https://docs.openmm.org/latest/userguide/theory/02_standard_forces.html>). Bonded terms are classed by arity: `HarmonicBondForce` takes 2 particles, `HarmonicAngleForce` 3, `PeriodicTorsionForce` 4. `NonbondedForce` selects its long-range method by enum: NoCutoff, CutoffNonPeriodic, CutoffPeriodic, Ewald, PME, LJPME. *Exceptions* handle 1-4 pairs, "always evaluated at full strength, regardless of the cutoff distance". **Transfer:** the method enum *is* the rung list of one term, and the kernel's range decides which rungs exist. Exceptions are the missing idea of a pair excluded from or rescaled in a non-bonded term because a bonded term already covers it. Today `NodeExclusion` repels edge-connected pairs that `EdgeSpring` is pulling together.
- **OpenMM `CustomCentroidBondForce`** (same page): the centre of each group is "a weighted average of the positions of all the particles in the group, with the weights being user defined". **Transfer:** P7c's grouped law has a precise ancestor. An outer energy over centroids reaches each member through the centroid map's weights, which is the chain rule (F9).
- **OpenMM `LocalEnergyMinimizer`** (<https://docs.openmm.org/latest/api-c++/generated/LocalEnergyMinimizer.html>). It runs L-BFGS on the *same* `System` the integrators step. Constraints are enforced "by adding a harmonic restraining force… steadily increased" until satisfied. **Transfer:** one term set, two realizations (integration and optimization), and an `ensure` constraint realized as a stiffening `encourage`. This is the hypothesis's best case, shipped.
- **OpenMM `MTSLangevinIntegrator`** (<https://docs.openmm.org/latest/api-python/generated/openmm.mtsintegrator.MTSLangevinIntegrator.html>). Forces go into groups with `setForceGroup()`, and the integrator takes `(group, substeps)` tuples "to evaluate the expensive, slowly changing forces less frequently". **Transfer:** a schedule combinator over terms. P5's "newest result for up to N steps" staleness ruling is already one, applied to a single term.
- **LAMMPS `pair_style hybrid` / `hybrid/overlay` / `hybrid/scaled`** (release 30Sep2026; <https://docs.lammps.org/pair_hybrid.html>). `hybrid` assigns "exactly one pair style to each pair of atom types". `hybrid/overlay` superimposes several "in an additive fashion". `hybrid/scaled` "adds a scale factor for each sub-style contribution". **Transfer:** *partition* and *weighted sum* are distinct, named combinators over a kind matrix. Kinds' matrix is a partition with one kernel; a weighted law list is `hybrid/scaled`.
- **HOOMD-blue** (7.1.2; <https://hoomd-blue.readthedocs.io/en/v5.2.0/hoomd/md/methods/brownian.html>). An `Integrator` holds several *methods*. Each method "applies the given equations of motion to a subset of particles" selected by a filter, and particles can be left fixed. **Transfer:** currencies can be *scoped*. Overdamped Brownian motion for one group beside Langevin dynamics for another is an existing composition, which answers part of the plan's "multi-integrator" question.
- **d3-force** (3.0.0; <https://d3js.org/d3-force/simulation>). "A force is a function that receives alpha", typically mutating velocities. Alpha decays from 1 toward `alphaMin` 0.001 at `alphaDecay` ≈ 0.0228 (about 300 ticks); `velocityDecay` is 0.4. `forceManyBody` is Barnes–Hut at θ 0.9, and a positive strength attracts. `forceLink`'s default strength is `1 / min(count(source), count(target))` because it "automatically reduces the strength of links connected to heavily-connected nodes". **Transfer:** a global schedule multiplier (alpha) separate from every term. The degree-normalized link default is an Em-style metric choice made for stability, and it is documented as one.
- **XPBD** (Macklin, Müller, Chentanez, MIG 2016, doi:10.1145/2994258.2994272). Compliant constraints remove "iteration count and time step dependent constraint stiffness", simulate "arbitrary elastic and dissipative energy potentials", and provide "constraint force estimates". **Transfer:** one parameter, compliance, spans `ensure` (zero) to `encourage` (positive). The constraint force estimate *is* a satisfaction report. Pins and anchors become two settings of one constraint term.
- **Box2D v3 and Solver2D** (Erin Catto, <https://box2d.org/posts/2024/02/solver2d/>). Eight solvers (PGS, TGS, NGS and XPBD variants) were run on one constraint set, and "TGS_Soft", with substeps and soft constraints, beat the others in nearly all tests. **Transfer:** rung comparison as a discipline. The same scene is run under each solver and measured, which is what P5a's tiled-kernel parity receipt already does for one term.
- **Lenia** (Chan, Complex Systems 28(3):251–286, 2019; <https://www.complex-systems.com/abstracts/v28_i03_a01/>) and **Particle Lenia** (Mordvintsev, Niklasson, Randazzo, 2022; <https://google-research.github.io/self-organising-systems/particle-lenia/>). Particle Lenia's particles "move against the local gradient of the *energy field* E=R−G", with no momentum. Its authors note that particles minimize their *own* energy, not the system's: "the global descent finds a much lower energy state that can't be sustained by the local update rule", and they prefer the local rule for "more interesting and diverse structures". **Transfer:** a law can be built from energies and still not descend a global objective. The global realization would be a different, duller picture. This is the clearest external evidence that the dynamics class is not a defect to be optimized away.
- **Non-reciprocal interactions** (Fruchart, Hanai, Littlewood, Vitelli, *Nature* 592:363–369, 2021; <https://www.nature.com/articles/s41586-021-03375-9>). Their dynamics "is not governed by an optimization principle", and non-reciprocity "leads to time-dependent phases", from flocking to synchronization. **Transfer:** the physics behind particle life's chasing. An asymmetric kind matrix places Kinds outside every energy model by construction (§5.7 F-e).
- **Kuramoto's Lyapunov structure** (van Hemmen and Wreszinski, *J. Stat. Phys.* 72:145–166, 1993; <https://link.springer.com/article/10.1007/BF01048044>). A Lyapunov function exists whose minimum is the phase-locked ground state. Against that, the Kuramoto–Sakaguchi equation "is not a gradient flow with respect to the Wasserstein distance" (Morales and Poyato 2019, <https://arxiv.org/abs/1908.07657>). **Transfer:** Sync has an objective, with caveats. Its minimizer on a connected graph is total synchrony, so the communities Sync shows are a transient property (§6).

### 4.2 Layout as optimization

- **Fruchterman–Reingold** (*SPE* 21(11):1129–1164, 1991; <https://www.reingold.co/force-directed.pdf>). Spring-electrical forces under a cooling temperature. **Transfer:** the baseline energy and the original schedule combinator.
- **Kamada–Kawai, stress majorization, SGD.** Kamada and Kawai (*IPL* 31(1):7–15, 1989) minimize stress one node at a time by Newton–Raphson. Gansner, Koren and North (GD 2004, "Graph Drawing by Stress Majorization") showed majorization "monotonically decreases the stress", with weights `d⁻²`. Zheng, Pawar and Goodman (*IEEE TVCG* 25(9):2738–2748, 2019; <https://doi.org/10.1109/tvcg.2018.2859997>) minimize the same stress by SGD over pairs, "faster and more consistently than majorization". **Transfer:** one objective with three optimizers in the literature, and a fourth, integration, in seiche's `StressSpring`.
- **Graphviz neato and sfdp** (16.1.0; <https://graphviz.org/docs/attrs/mode/>, <https://graphviz.org/docs/layouts/sfdp/>). neato's `mode` picks the optimizer for one stress objective: `major` (the default), `KK`, `sgd`. Two further modes add structure: `hier`, "top-down directionality similar to dot", and `ipsep`, minimum separations. sfdp is "a fast, multilevel, force-directed algorithm" after Hu (2005), with a quadtree. **Transfer:** a production tool exposes *realization* as a parameter and *constraint terms* as modes over one objective. Stress plus depth is a shipped, named composition.
- **FM³** (Hachul and Jünger, GD 2004) and the **OGDF `ModularMultilevelMixer`** (OGDF 2023.09 "Elderberry"; <https://ogdf.github.io/doc/ogdf/classogdf_1_1_modular_multilevel_mixer.html>). FM³ coarsens by "solar systems" (sun, planet, moon) and computes repulsion by multipole expansion. OGDF's mixer takes four exchangeable modules: a multilevel builder (default `SolarMerger`), an initial placer (`BarycenterPlacer`), a one-level layout (`FastMultipoleEmbedder`), and an optional final layout. **Transfer:** coarsening is its own module type, separate from the force law that runs on each level. P7c's grouped law is one level of this, with meaning as the merger.
- **ForceAtlas2** (Jacomy, Venturini, Heymann, Bastian, *PLoS ONE* 9(6):e98679, 2014; <https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0098679>). Repulsion is "proportional to the produce of the degrees plus one", and the force exponents are classed as FR "(2,−1)", ForceAtlas2 "(1,−1)", LinLog "(0,−1)". LinLog "results in a placement that corresponds to Newman's modularity". **Transfer:** the kernel exponent is a parameter, and naming it exposes the §5.7 F-a finding.
- **LinLog and modularity** (Noack, *Phys. Rev. E* 79:026102, 2009; <https://doi.org/10.1103/PhysRevE.79.026102>). Energy models of pairwise attraction and repulsion subsume modularity: "Layouts with optimal energy are relaxations of… clusterings with optimal modularity". **Transfer:** the carried inference of the Energy law has a theorem behind it, but only at LinLog's exponent.
- **WebCoLa and IPSep-CoLa** (<https://ialab.it.monash.edu/webcola/>; Dwyer, Koren, Marriott, *IEEE TVCG* 12(5):821–828, 2006). Constraint-based layout supports alignments, flow layout for directed graphs, non-overlap and grouping, with separation constraints solved inside stress majorization. **Transfer:** constraints as first-class terms beside the energy. The adoption plan already took WebCoLa's silent-soft failure as its caution (adoption Findings, first row).
- **SetCoLa** (Hoffswell, Borning, Heer, EuroVis 2018; <https://github.com/uwdata/setcola>). Sets are defined by predicates, and alignment, position, order, circle, cluster, hull and padding constraints apply to sets; it compiles to WebCoLa. **Transfer:** scope as selection, so one authored spec reapplies to a second graph. The projection grammar's second-dataset receipt applies to dynamics specs unchanged.
- **Penrose** (Ye et al., SIGGRAPH 2020; <https://penrose.cs.cmu.edu/media/Penrose_SIGGRAPH2020a.pdf>). `ensure` becomes "a constraint that the solver seeks to enforce exactly", and `encourage` becomes "an objective that the solver seeks to minimize". It is solved by an exterior-point method with L-BFGS. **Transfer:** the satisfaction vocabulary the projection grammar adopted (catalog line 434), with its solver. `ensure` is realized as a penalty schedule, as in OpenMM's minimizer.
- **Davidson–Harel annealing** (*ACM TOG* 15(4):301–331, 1996; <https://dl.acm.org/doi/10.1145/234535.234538>). The energy sums node distribution, borderlines, edge lengths, *edge crossings* and node–edge distances, with adjustable weights. **Transfer:** annealing earns its place where a term is discrete or non-differentiable (crossings), which integration cannot carry. seiche's `Anneal` omits crossings (§5.7 F-b).
- **Magnetic spring model** (Sugiyama and Misue, *J. Visual Languages and Computing* 6:217–231, 1995, doi:10.1006/jvlc.1995.1013). Edges are springs and compass needles, and the model "can control orientations of links". **Transfer:** Flow's ancestor. The needle is a separate term from the spring.
- **Spectral layout** (Hall, *Management Science* 17(3):219–229, 1970; Koren, *Computers & Mathematics with Applications* 49:1867–1888, 2005; <https://doi.org/10.1016/j.camwa.2004.08.015>). The Laplacian eigenprojection is "a solution to a minimization problem". **Transfer:** the one arrangement in the picker that is the closed-form minimizer of an energy (§6.2).
- **(SGD)²** (Ahmed, De Luca, Devkota, Kobourov, Li, 2021; <https://arxiv.org/abs/2112.01571>). It optimizes "any criterion that can be described by a differentiable function", including ideal edge lengths, stress, neighbourhood preservation, node resolution, angular resolution and aspect ratio. **Transfer:** a weighted multi-criterion layout as one optimizer over several energy terms. It is prior art for P7d once a common scale exists.
- **t-SNE, UMAP, and the attraction–repulsion spectrum.** Böhm, Berens and Kobak (*JMLR* 23, 2022; <https://arxiv.org/abs/2007.08902>) show that neighbour embeddings "combine an attractive force between neighboring pairs of points with a repulsive force between all points". Varying the balance "yields a spectrum": UMAP is t-SNE with increased attraction, and "ForceAtlas2… corresponding to t-SNE with the attraction increased even more". **Transfer:** the `semantic.embedding` arrangement's producers and the Energy law lie on one family with one parameter. It is the strongest external case for the hypothesis.
- **Density-equalizing maps** (Gastner and Newman, *PNAS* 101(20):7499–7504, 2004, <https://www.pnas.org/doi/abs/10.1073/pnas.0400280101>; Gastner, Seguy, More, *PNAS* 115(10):E2156–E2164, 2018, <https://doi.org/10.1073/pnas.1712674115>). The diffusion cartogram advects tracers by the density flux. The 2018 flow-based algorithm reaches the same kind of result with "numerically easier-to-solve equations of motion". **Transfer:** Density already has two realizations in the literature.
- **Boids** (Reynolds, SIGGRAPH 1987, "Flocks, herds and schools: a distributed behavioral model"). **Transfer:** the canonical dynamics-only law.

### 4.3 Objectives and their geometry

- **Energy-based models** (LeCun, Chopra, Hadsell, Ranzato, Huang, 2006; <http://yann.lecun.com/exdb/publis/pdf/lecun-06.pdf>). "Inference consists in clamping the value of observed variables and finding configurations of the remaining variables that minimize the energy". **Transfer:** the model is the energy, and the inference procedure is separate and swappable. Clamping is a pin: a pinned body stays in the energy and is never moved, which is exactly the 2026-10-01 Anneal fix (`seiche/laws/anneal.rs:145-154`).
- **Wasserstein gradient flows** (Jordan, Kinderlehrer, Otto, *SIAM J. Math. Anal.* 29(1):1–17, 1998; Carrillo, McCann, Villani, *Rev. Mat. Iberoamericana* 19(3):971–1018, 2003, <https://ems.press/journals/rmi/articles/5018>). The Fokker–Planck equation is "a gradient flux or steepest descent for the free energy with respect to the Wasserstein metric". The granular-media line extends this to nonlocal interaction energies. **Transfer:** Density is an energy term after all, in a different metric, and that metric fixes what "adding a force to Density" means (§7.2).

## 5. Decomposition

### 5.1 The laws (12)

The law builders are `pictograph/physics_catalog.rs:1012-1071`. Density is on `density-cpu`.

| Law (id) | Terms (file:line) | Topology | Kernel | State | Currency | Class | Signature observable |
|---|---|---|---|---|---|---|---|
| Springs (`spring.rapier`) | `NodeExclusion` `seiche/forces.rs:42`; `EdgeSpring` `:143`; `Boundary` `:196`; rapier contacts | all-pairs within cutoff 1 000; edges; unary; contact pairs | `s/d²`, floor 8, `s` 220 000 (`:51-63`), energy `s/d` truncated; `k(d−L)`, `k` 10, `L` 170 (`:151-157`); `−b·x`, `b` 0.08 (`:202-208`) | position | force | E, plus a contact constraint | ring-down gone, no overlaps, spread ≤ 250 (P2 receipt) |
| Charge (`charge.barnes-hut`) | `BarnesHutRepulsion` `seiche/barnes_hut.rs:89` at 6 000 (`pictograph/physics_catalog.rs:62-65`); `EdgeSpring`; `Boundary` | all-pairs, quadtree at θ 0.5 (`barnes_hut.rs:42-48`) | `s·m/d`, energy `−s ln d`; Barnes–Hut approximates its gradient | position | force | E | no overlaps |
| Stress (`stress.kamada-kawai`) | `NodeExclusion`; `StressSpring` `seiche/laws/stress.rs:30`; `Boundary` | every connected pair, distances by Dijkstra at cost `1/multiplicity` (`physics_catalog.rs:994-1009`) | `(k/h²)(d − hL)`, `k` 40 (`stress.rs:83-108`) | position | force | E (Kamada–Kawai stress plus repulsion and centre) | stretch ≥ 2.5 |
| Energy (`energy.linlog`) | `NodeExclusion`; `LinLogForce` `seiche/laws/linlog.rs:27` | all-pairs; edges; unary | `r·w_i·w_j/d`, `w` = deg+1; `a·d` along edges; `g·w_i·x` (`:52-102`) | position | force | E: ForceAtlas2's (1, −1), not LinLog's (0, −1) (F-a) | no overlaps |
| Orbit (`orbit.gravity`) | `NodeExclusion`; `Gravity` `seiche/laws/gravity.rs:27` | all-pairs | softened acceleration `G·m_j·Δ/(d²+ε²)^{3/2}` (`:116-126`); a once-only tangential velocity write (`:80-109`); counter-damping drive `m·γ·v` (`:131-135`) | position, velocity | force, plus one velocity write | H: Plummer gravity in the mass metric with damping cancelled; the minimizer is collapse (*Reading*) | energy ≥ 1 at 1 s and 6 s |
| Kinds (`kinds.particle-life`) | `NodeExclusion`; `ParticleLife` `seiche/laws/particle_life.rs:32` | all-pairs within 220; kind matrix | core repulsion, then a tent × `K[kind_i][kind_j]` (`:92-100`), *i's row only* (`:121-123`); centring | position | force | N, because the matrix is asymmetric (F-e) | energy ≥ 1 at 1 s |
| Flock (`flock.boids`) | `NodeExclusion`; `Boids` `seiche/laws/boids.rs:26` | all-pairs within 60 (separation); edge-mates (alignment, cohesion); unary (cruise, centre) | `s(1−d/R)/d`; `(v̄−v)·α`; `(x̄−x)·c` averaged over each node's own mates; `(v₀−\|v\|)·v̂/2` (`:84-118`) | position; reads velocity | force | N (an active cruise drive; mate averaging is non-reciprocal) | energy ≥ 1 at 1 s and 6 s |
| Sync (`sync.kuramoto`) | `NodeExclusion`; `Kuramoto` `seiche/laws/kuramoto.rs:31` | edges (phase); unary (ring target) | `θ̇ = ω + (K/deg)·Σ sin(θ_j − θ_i)` (`:89-105`); Hooke toward `r(θ)` (`:107-122`) | phase (interior, in a `Mutex`), then position | force (the phase steps inside `apply`) | H: phase descends `−Σ cos` in a degree metric when ω is uniform; the minimizer is total synchrony (*Reading*) | two communities end in two phase clusters (P1 test); energy ≥ 1 at 1 s |
| Flow (`flow.magnetic`) | `NodeExclusion`; `MagneticSpring` `seiche/laws/magnetic.rs:27`; `Boundary` | all-pairs; directed edges | `s/d²`; `k(d−L)`; needle `τ·min(d, 2L)/2·(f − Δ̂)` (`:99-104`) | position | force | E + N (the needle as written is not a gradient; F-d) | a directed chain ends monotone along the field (P1 test) |
| Anneal (`anneal.davidson-harel`) | `Anneal` `seiche/laws/anneal.rs:28` | all-pairs; edges; unary | energy `Σ r/d + g\|x\|² + Σ(d−L)²` (`:69-91`); Metropolis moves, cooling 0.995 from 4 000 to 1 (`:143-170`) | position, written (`:165`) | kinematic | K realizing an E objective with Springs' functional form (F-b) | energy ≤ 5 |
| Still (`still.default`) | `Hold` `seiche/laws/hold.rs:22` | unary | `v ← 0` each tick (`:24-33`) | velocity, written | kinematic | K: a projection. With anchors it becomes an overdamped descent of the anchor energy, with step `dt²/m` (*Reading*) | holds the drop point within 5 px (P4) |
| Density (`density.gastner-newman`, branch) | `Density` `1bf47cf9:crates/conatus/seiche/src/laws/density.rs:294`; medium seam `DensityMedium` (`:50`) | medium: a grid with cloud-in-cell splat, implicit diffusion and CFL substeps | `v = −D·∇ρ/ρ` (`:7-15`) | field, then position, written (`:469`) | kinematic; resident in P6b | K realizing a Wasserstein gradient flow of entropy (*Reading*, via JKO) | mass/area Spearman ≥ 0.8; density CV |

### 5.2 The overlays (8)

The overlay builders are `pictograph/physics_catalog.rs:1075-1108`; the catalog is `CANVAS_PHYSICS_OVERLAYS` (`:431-440`).

| Overlay (label) | Term (file:line) | Topology | Kernel | Source | Class |
|---|---|---|---|---|---|
| `degree-repulsion` (Hub room) | `DegreeRepulsion` `seiche/overlays/degree_repulsion.rs:25` | all-pairs within 400 | push on j = `s·w_i/d`, *the source's weight only* (`:88`) | mass: ln(deg+1) or PageRank | Em (non-reciprocal; F-c) |
| `domain-cluster` (Group pull) | `DomainCluster` `seiche/overlays/domain_cluster.rs:23` | groups | `s(c_g − x)` (`:65`) | site only (`physics_catalog.rs:1087`; F-g) | E: within-group variance |
| `hub-gravity` (Hub pull) | `HubGravity` `seiche/overlays/hub_gravity.rs:23` | all-pairs | pull toward j = `s·w_j/d` (`:76`) | mass | Em (F-c) |
| `depth-gravity` (Depth) | `DepthGravity` `seiche/overlays/depth_gravity.rs:23` | unary | `s(depth·h − x·â)·â` (`:62`) | depth: roots, layers or focus | E |
| `grid-snap` (Grid) | `GridSnap` `seiche/overlays/grid_snap.rs:19` | unary | `s(round(x/c)·c − x)` (`:44`) | none | E (non-convex, piecewise) |
| `gravity-locus` (Centre) | `GravityLocus::at` `seiche/overlays/gravity_locus.rs:25` | unary | `s(p − x)` (`:82`) | none | E |
| `tide` (Tide) | `GravityLocus::tidal` (same file) | unary | `s(p(t) − x)`, with `p(t)` on a sine | none | H: a time-dependent potential; never rests |
| `skeleton` (Skeleton) | `StressSpring` over spanning-tree edges at one hop, `k` 60 (`physics_catalog.rs:61`, `:1100-1106`) | tree edges | `k(d − L)` | `min_spanning_tree` | E |

### 5.3 Slots and always-on terms

The tick applies forces in this order: law and overlays, then couplings, affinity and anchor, all in one reset window (`seiche/lib.rs:723-775`).

| Slot or term | Where | Topology | Kernel | Currency | Class |
|---|---|---|---|---|---|
| Anchor (arrangement pull) | `AnchorSpring` `seiche/anchor_force.rs:42`; installed by `sync_anchor_force` `pictograph/strategy.rs:208`; stiffness `arrangement_pull`, saved in `SavedSceneV1` | unary, per node toward its slot | `k(\|o\| − slack)·ô`, slack 0.5 (`:83-105`) | force | E. An encourage-class *target* term |
| Affinity | `AffinitySpring` `seiche/affinity_force.rs:55`; `sync_affinity_force` `pictograph/strategy.rs:576` | pair list: structural Jaccard, content, or their noisy-OR (`pictograph/../canvas.rs` `AffinityBlend`, lines 75–97) | `k·w·(d − L)` only when `d > L` (`:91-122`) | force | E (a one-sided spring) |
| Couplings (fields) | `CouplingForce` `seiche/coupling_force.rs:38`; responses `numen/coupling.rs:69` | unary over a numen field, scoped by `NodeSelector` | `AttractToMin` / `RepelFromMax`: `∓s∇φ`; `ContainmentWall`: `s·φ·∇φ` inside; `DampenInside`: `v ×= f`; `AlignVelocity`: `v ← s·u(x)`; `FlowAdvect`: `x += s·u·dt`; `Open`: nothing (`:102-192`) | force for the first three; kinematic for the next three | E for the gradient and wall responses; dissipative for damping; K for align and advect, which are E only when `u = −∇φ` |
| Contacts | rapier colliders, one per node | contact pairs | non-penetration | inside the integrator | a constraint, softly solved; read afterwards as `LayoutStats.overlaps` |
| Damping | `DEFAULT_LINEAR_DAMPING` 2.5 (`seiche/lib.rs:218`) | unary | `−γ·v` | inside the integrator | dissipation: what makes an E law descend |
| Pins (drag) | `Simulation::pin` (`seiche/lib.rs:829-838`): the body becomes `KinematicPositionBased` | unary | `x ← target` | kinematic | an ensure-class constraint. No satisfaction is reported |
| Scene field | `SceneField::Vortex` (`seiche/scene_spec.rs:170`); scene bodies only, not nodes | unary | tangential `s` plus inward pull | force | N (a curl) + E |
| Repulsion solver seam | `RepulsionSolver` (`seiche/lib.rs:364`), threshold (`:373`) | — | must carry `NodeExclusion`'s complete law, `RepulsionRequest` (`:262-271`) | — | a *rung*, not a term |

### 5.4 The arrangements

The picker lists eight in `CANVAS_LAYOUT_STRATEGIES` (`pictograph/cartography_scene.rs:146-161`). `radial.default` is dispatched (`:273`) but not listed. `semantic.embedding` exists in cartography only (`cartography/adapters/mod.rs:243`).

Every arrangement is a graph-side *producer* that discloses per-item values, plus a portable `sceno::Arrangement` that `scenomise` solves ("Nothing here places anything", `cartography/adapters/mod.rs:6-11`). That is already the sources-as-channels split.

| Id (label) | Producer (disclosure) | `sceno` arrangement | Objective? | Law-form twin today |
|---|---|---|---|---|
| `phyllotaxis.default` (Spiral) | ordinal by recency (`cartography_scene.rs:197-199`) | `Spiral` (golden angle, `sceno/score.rs:179`) | none on the graph: generated slots, assigned in order | none |
| `grid.default` (Grid) | ordinal | `Grid` | none: a lattice of slots, assigned in order | Grid overlay, but it picks the *nearest* cell, not the ordinal one |
| `spectral.default` (Spectral) | `spectral_coords`: power iteration on `cI − L`, two smallest non-trivial eigenvectors (`cartography/adapters/producers.rs:83`, `:139`) | `Embedded` | yes. The closed-form minimizer of Hall's energy `Σ a_ij\|x_i − x_j\|²` under centring and unit norm (Koren 2005) | none exact; Stress is the nearest law (*Reading*) |
| `penrose.default` (Penrose) | ordinal | `Penrose` (aperiodic tiling) | none: a generator | none |
| `lsystem.default` (Fractal) | ordinal | `LSystem` | none: a generator | none |
| `kanban.default` (Columns by site) | categorical axis = site (`cartography_scene.rs:204`) | `Kanban` | a constraint: `x = column(site)`, packed within each column | Group pull by site: a centroid rather than a column |
| `kanban.community` (Columns by cluster) | Louvain partition (`:226`) | `Kanban` | the same constraint | none: Group pull reads site only (F-g) |
| `timeline.default` (Timeline by order) | numeric axis = enumeration order (`:257`) | `Timeline` | a constraint: `x = f(order)`, packed | none: no time depth source |
| `radial.default` (not listed) | breadth-first rings from the focus (`cartography/adapters/mod.rs:345`) | `Radial` | a constraint: `r = ring·h`, packed by angle | Depth from focus, but on a straight axis, not a radial one |
| `semantic.embedding` (cartography only) | a host-run UMAP, t-SNE or PCA | `Embedded` | the producer's own objective | Affinity on content pairs is the nearest (*Reading*) |

### 5.5 Sources

| Channel | Options today | Read by | Arrangement counterpart |
|---|---|---|---|
| kind | site, cluster, colouring, island, degree (`physics_catalog.rs:443-449`), folded to at most 8 (`:824-866`) | Kinds | categorical axis (Columns) |
| mass | degree, PageRank (`:452-453`, `:783-822`) | Orbit, Hub room, Hub pull; Density on the branch | `weight` disclosure (Radial's weighted policy) |
| depth | roots, layers, focus (`:456-460`) | Depth | numeric axis (Timeline, Radial rings) |
| groups | site only (`:719`, `:1087`) | Group pull | categorical axis |
| pairs | structural Jaccard, content, blend (`pictograph/strategy.rs:520-616`) | Affinity | — |
| distances | weighted Dijkstra (`physics_catalog.rs:994`) | Stress, Skeleton | `embedding` (Spectral) |

Sources are snapshotted at build and rebuilt on topology change (`physics_catalog.rs:169`, `:242-249`, `:1298-1323`). The plan ruled "Snapshots only for now" for meaning. Laws and arrangements read the same graph facts through two disclosure vocabularies; F3's T2 makes them one.

### 5.6 Where the decomposition is not clean

1. **Sync** keeps a phase state inside the `Force` (a `Mutex`) and draws it as position with a spring. The force contract has no place for non-positional state. The grammar needs *state = phase* plus an *encoding* from phase to position. That encoding is the projection grammar's layer (catalog lines 159–171) reached from the dynamics side (*Reading*).
2. **Orbit** bundles three things: an initial condition (the one-time velocity kick), a conservative kernel, and an anti-dissipation drive. As terms they would be one initial condition, one E kernel under the mass metric, and one drive that cancels damping.
3. **Flow** bundles repulsion, a spring and a needle in one `Force`. The needle alone is non-conservative (F-d).
4. **Boids'** alignment reads velocity. That term is over (position, velocity), so a positions-only term type cannot hold it.
5. **Anneal, Density and Hold** are not kernels at all. They are realizations (an optimizer, a transport, a projection) and carry their objective implicitly. In a grammar they become a *realization* that names the energy terms it drives (*Reading*).
6. **Charge's** id names its rung. The donor's own spec called Barnes–Hut "a scaling implementation choice, not a new user-facing semantics model" (plan, Findings, 2026-09-02). Charge is still a distinct law, because its kernel is `1/d` against `NodeExclusion`'s `1/d²`, so the id fuses kernel and rung.
7. **Damping and contacts** live in rapier, not in the force list, so the spec must name terms that seiche does not expose as `Force`s.

### 5.7 Findings from the decomposition (2026-10-02)

- **F-a. Energy's kernel is ForceAtlas2's, not LinLog's.** `LinLogForce` applies attraction `a·d` and repulsion `r/d` (`seiche/laws/linlog.rs:62-96`). Its doc reads "Noack's LinLog energy model attracts along edges *linearly* in distance… as forces, attraction `a·d`" (`:9-11`). LinLog's *energy* is linear in distance, so its attractive *force* is constant: the force model (0, −1). The implemented force model is (1, −1), which Jacomy et al. assign to ForceAtlas2 and place "between Noack's LinLog and the algorithm of Fruchterman and Rheingold". With `degree_weighted` on, the default, the law *is* ForceAtlas2's model. With it off, it is not LinLog. Noack's modularity correspondence attaches to LinLog's exponent, so the catalog's "communities as islands" is ForceAtlas2-strength separation. The P1 test, two cliques further apart than under Springs, still holds, as it should between (2, −1) and (1, −1).
- **F-b. Anneal's energy has Springs' functional form.** Its energy is `Σ r/d + g|x|² + Σ(d − L)²` (`anneal.rs:69-91`). Springs' forces integrate to the same three terms: `NodeExclusion`'s `s/d²` force is the gradient of `s/d`, `EdgeSpring`'s is `(k/2)(d − L)²`, and `Boundary`'s is `(b/2)|x|²`. Only the constants differ: Anneal has `r` 500 000, edge weight 1, `L` 120, `g` 0.01 (`:44-56`); Springs has 220 000, 5, 170, 0.04. Anneal also has no cutoff and no contacts. Davidson–Harel's distinguishing term, edge crossings, is absent. So Anneal is a different *realization* (Metropolis annealing) of a re-tuned spring-electrical objective, not a different objective.
- **F-c. The hub overlays are non-reciprocal.** `DegreeRepulsion` pushes `j` by `s·w_i/d` and `i` by `s·w_j/d` (`degree_repulsion.rs:88`). `HubGravity` pulls `i` toward `j` by `s·w_j/d` (`hub_gravity.rs:76`). The accelerations equal those of a reciprocal pair potential with inertial mass `w`: two-dimensional gravity, energy `−s·w_i·w_j·ln d`. So each overlay is a gradient flow in the metric `diag(w)`. Nodes of degree zero have `w = ln 1 = 0` and are test particles. Summed with terms under the identity metric (every other term; rapier weighs every node body about the same, plan Findings 2026-09-02 P1), the total is not guaranteed to descend anything (*Reading*; §6.1).
- **F-d. Flow's needle is not a gradient as written.** The needle force on `j` is `τ·min(d, 2L)/2·(f − Δ̂)` (`magnetic.rs:101-102`). Without the length factor, `τ(f − Δ̂)` is the negative gradient of `τ(|Δ| − Δ·f)`, which is zero exactly when the edge points along the field. Multiplying a gradient field by a non-constant `d/2` breaks the symmetric Jacobian, so the needle as written has a curl (*Reading*; derivation in §6.1). A conservative variant is a one-line change. Whether it reads the same is a receipt question (F8).
- **F-e. Kinds is non-conservative exactly because its matrix is asymmetric.** `ParticleLife::seeded` draws `K[a][b]` and `K[b][a]` independently (`particle_life.rs:73-82`), and the force on `i` uses `i`'s row only (`:121-123`). With `K` symmetric, every pair force would be central and reciprocal, so it would be a pair potential and the law would be class E. Chasing would then give way to equilibrium demixing (*Reading*). This is the Fruchart et al. distinction inside one parameter.
- **F-f. `never_rests` excludes Kinds.** `PhysicsLaw::never_rests` lists Orbit, Flock and Sync (`physics_catalog.rs:160-165`) and has since P1 (`37477457`). The plan's 2026-09-02 finding says "Orbit, Kinds, Flock, Sync and the oscillating locus never rest" and that this "landed as `PhysicsLaw::never_rests`" (plan, Findings). The P2 receipt reads Kinds' energy at 1 s only, where Orbit and Flock are read at 1 s and 6 s (`Code/testing/mere/physics_p2_receipt.md`, the results table). A switch to Kinds therefore settles for the 360-tick budget (`pictograph/../canvas.rs`, `SETTLE_TICKS`, line 203) and then stops ticking on the web (F10).
- **F-g. Group pull reads site only.** The overlay builder passes `self.site_groups()` (`physics_catalog.rs:1087`), while the kind source offers five channels. So "Columns (by cluster)" has no law-form twin. P7b's "Meaning … feeds … `DomainCluster` groups" also needs Group pull to take a group channel first.
- **F-h. mer3ly's citation no longer carries physics.** At `7e455f9`, `sceneState()` emits `placement` (as `HeldPlacement`), `selection`, `mer3ly.motion` and `mer3ly.backdrop` delta sections (`repos/mer3ly/assets/graph-sandbox.js`, lines 1535–1558). The 2026-08-16 survey's `physics` field, then "absent from the contract" (survey, wire table), is gone. The motion class (anchored or free) is the one dynamics fact the site still shares. It is an anchor-pull setting, not a law.
- **F-i. Rapier's role in the realization is fixed today.** Every law except the three state-writers is integrated by rapier with damping 2.5 and contacts. The P5 rulings keep rapier as "the fallback in any case" (plan, "The repulsion lag"), and the resident mode drops contacts and joints for the laws it serves. So *realization* is already a two-option axis per law, and it is about to become three (*Reading*).

## 6. The hypothesis test

### 6.1 Method

For each element, three questions:

1. Is there a scalar objective whose descent, or exact solution, produces the element's output?
2. What realizations exist or could exist: closed form, optimizer, integration?
3. Does the *minimizer* carry the element's inference, or does the *trajectory*?

The derivations behind the classes are standard; they are recorded here so they can be checked (*Reading, not ruled*).

- **When forces are a gradient.** A force field `F(x)` is a gradient exactly when its Jacobian `∂F_a/∂x_b` is symmetric. A pair force that is central and reciprocal (`F_ij = −F_ji`, along `x_i − x_j`, magnitude a function of `d`) is always a gradient. A pair force weighted by one side only is a gradient in the metric `diag(w)`.
- **Flow's needle** (F-d). With `Δ = x_j − x_i`, the needle force on `j` is `g(d)·(f − Δ/d)`, where `g` is the length factor. Its Jacobian is `g'(d)·(f − Δ̂)⊗Δ̂ − (g/d)(I − Δ̂⊗Δ̂)`. The second term is symmetric. The first is symmetric only when `f ∥ Δ̂` or `g' = 0`.
- **Kuramoto.** With uniform ω, in the rotating frame, `θ̇_i = (K/deg_i)·Σ_j sin(θ_j − θ_i) = −(1/deg_i)·∂V/∂θ_i`, with `V = −K·Σ_edges cos(θ_i − θ_j)`. That is a gradient flow in the metric `diag(deg)`, and `V`'s global minimum on a connected graph is equal phases.
- **Density.** The heat equation written as `∂ρ/∂t + ∇·(ρv) = 0` has `v = −D∇ρ/ρ = −D∇ log ρ`. That is the Wasserstein-2 gradient of the entropy `D∫ρ log ρ` (JKO). Its minimizer under walls and conserved mass is uniform density, which is the cartogram.
- **Hold.** Velocity is zeroed before the step, so each tick moves a body by `dt²·F/m` from the forces added in the same tick: gradient descent with a tiny step.

### 6.2 Results

| Element | Objective | Realizations: in tree / in the literature | Inference carried by | Verdict |
|---|---|---|---|---|
| Springs | spring-electrical energy with cutoff, plus contacts | integration / FR cooling, L-BFGS | minimizer | fits |
| Charge | spring-electrical energy with log repulsion | integration with Barnes–Hut / FM³, sfdp | minimizer | fits |
| Stress | Kamada–Kawai stress, plus repulsion and centre | integration / Newton (KK), majorization, SGD, neato's modes | minimizer | fits |
| Energy | (1, −1) energy, degree-weighted | integration / FA2 adaptive speed | minimizer | fits; kernel finding F-a |
| Anneal | Springs' functional form | Metropolis / L-BFGS, integration | minimizer | fits, as a *realization* (F-b) |
| Density | entropy, Wasserstein metric | grid transport, CPU (branch) / GPU (P6b), the flow-based algorithm of 2018 | minimizer (uniform density) | fits, in another metric |
| Still | none of its own | projection | the arrangement | a realization, not a term |
| Orbit | Plummer energy in the mass metric | integration only | trajectory; the minimizer is collapse | objective exists, meaning in the dynamics |
| Sync | `−Σ cos` on phase, degree metric | integration only | the transient; the minimizer is total synchrony | objective exists, meaning in the dynamics |
| Kinds | none (asymmetric matrix) | integration only | trajectory | no objective (symmetric `K` would make one) |
| Flock | none (active cruise drive) | integration only | trajectory | no objective |
| Flow | spring part yes; needle no | integration | minimizer for the spring part | partly fits; a fixable near-miss |
| E overlays (Group pull, Depth, Grid, Centre, Skeleton) | yes | integration | minimizer | fit |
| Em overlays (Hub room, Hub pull) | yes, in their own metric | integration | minimizer, alone | fit alone, not in a mixed sum |
| Tide | a time-dependent potential | integration | trajectory | dynamics |
| Anchor, Affinity | yes | integration | minimizer | fit; the anchor is the target bridge |
| Spectral | Hall's energy | closed form (power iteration) | minimizer | fits, as a closed form |
| Columns ×2, Timeline, Radial | constraint satisfaction plus packing | closed form | the constraint | fit as targets, not energies |
| Spiral, Grid, Penrose, Fractal | trivial: assignment by ordinal | closed form | the generator | no objective worth the name |
| `semantic.embedding` | the producer's: t-SNE or UMAP cross-entropy | host producer / the attraction–repulsion spectrum | minimizer | fits |

**Counts.**

- **Laws (12).** Six fit (Springs, Charge, Stress, Energy, Anneal as a realization, Density in its own metric). Two have an objective whose meaning is in the dynamics (Orbit, Sync). Three have no objective, or only partly (Kinds, Flock, Flow's needle). One is a projection (Still).
- **Overlays (8).** Five E, two Em, one time-dependent.
- **Listed arrangements (8).** One closed-form energy minimizer (Spectral), three constraint arrangements (two Columns, Timeline) and four generators (Spiral, Grid, Penrose, Fractal). Outside the picker, Radial is a constraint and `semantic.embedding` fits.

### 6.3 Where it fits: one objective, many realizations

The fit is not hypothetical. The tree already holds three cases.

- **Spectral and `semantic.embedding`.** Cartography records that "Spectral and semantic embedding are one arrangement now" (`cartography/adapters/parity.rs`, line 466): one `Embedded` arrangement, two producers. One producer is a closed-form energy minimizer. The other is an optimizer on a neighbour objective, which Böhm et al. place on one spectrum with ForceAtlas2, which is the Energy law.
- **Anneal and Springs.** One functional form, two realizations (F-b).
- **The anchor spring.** It turns any arrangement into an encourage-class energy term at stiffness `k`, and pins make it ensure-class. That is XPBD's compliance axis, already wired through `arrangement_pull`.

Outside the tree, OpenMM runs one `System` under integrators and under L-BFGS. Graphviz runs one stress objective under three optimizers. Particle Lenia shows an energy-built law whose global realization is a different picture.

### 6.4 Where it fails, and the verdict

The objective model fails in two places, for different reasons.

- **The dynamics class.** Kinds, Flock and Flow's needle have no objective. Orbit and Sync have one, but its minimizer is the wrong picture: collapse, and total synchrony. For these, integration is the only realization, and the receipt must say *when* to look. P2 already does this: Orbit and Flock are read at 1 s and 6 s, Sync at 1 s.
- **The generator arrangements.** Spiral, Grid, Penrose and Fractal assign items to generated slots by ordinal. An objective can be written down (minimize the assignment cost of ordinal against slot), but it carries nothing; the inference is in the generator.

**Verdict** (*Reading, not ruled*). The hypothesis as stated, "two realizations of one objective model", holds for the energy class and fails for the rest. What holds across all elements is a *specification* model with three kinds of term:

- **Energy terms** (E, Em). Closed form, optimizer or integration can realize them. They add meaningfully under one metric, and the minimizer is the picture.
- **Dynamics terms** (H, N). Only integration realizes them. The picture is a trajectory property, so the spec carries the horizon and the observable.
- **Target terms.** A slot set from a generator (an arrangement, with its `Score` as its portable form), held at a satisfaction class: encourage (the anchor) or ensure (pins and holds).

Arrangements that are energy minimizers (Spectral, `semantic.embedding`) or constraint solutions (Columns, Timeline, Radial) are closed-form realizations of energy or constraint terms, so they have law-form twins in principle. Generators do not, and do not need them.

### 6.5 What it means for the model

1. **Optimizer realizations are legal only for all-energy compositions under one metric.** A picker can offer "settle by majorization" for Stress plus Depth, never for Orbit plus anything. The class decides; the currency alone does not.
2. **The mass channel is a metric, not only a weight.** Orbit's masses, the hub overlays' weights and Kuramoto's `1/deg` each choose a metric. "Shared or independent scale resolution" (sketch item 2) applies to the metric as much as to the strength. Two terms add as energies only if they share one (*Reading*).
3. **A composition that includes a dynamics term is a dynamics composition.** Its receipt is a trajectory observable at a stated time.
4. **Arrangements do not need to become energies.** The spec references them as targets, and the `Score` stays their record. This keeps the seam doc's "arrangements compute, physics simulates" intact: computing produces targets, and simulating realizes terms.
5. **A term's class is checkable at runtime**, so it can be declared and verified rather than asserted. Two tests suffice: an energy-descent test (under overdamped flow, the declared energy must not increase) and a reciprocity test (the forces must sum to zero for a closed pair set). A class-N term must *fail* the descent test, which is the positive control the instrument needs.

## 7. Combinator semantics

### 7.1 By currency and class

| Combinator | All E, one metric (force) | Mixed metric, or any H/N term (force) | With a kinematic law | With a resident law |
|---|---|---|---|---|
| **Weighted sum** | an energy sum; any realization; weights need a common scale (F5) | a vector-field sum; integration only; no objective | only through conversion into the law's currency: forces become velocity, `v = F/γ` (ruled). For Density this is the free-energy sum (§7.2) | only terms with resident kernels or the lagged upload (ruled) |
| **Scope / partition** | an energy over the scoped pairs; legal | legal as dynamics | per-subset integrator methods (the HOOMD model), which need disjoint scopes (*Reading*) | the scope must be expressible as a GPU mask |
| **Groups** (scope plus coarsening) | outer energy on centroids, weight-shared to members, is a gradient (F9) | legal as dynamics | the centroid force becomes a kinematic velocity | the centroid reduction needs a kernel |
| **Sequence / schedule** | continuation: each stage starts at the last one's minimum; capture-as-anchor turns stage `k`'s result into a target for stage `k+1` | stages with horizons | a stage boundary is the only safe point to switch currency | per stage, with readback at the boundary |
| **Level of detail (rung)** | the same term; the rung must reproduce the law to a tolerance | the same | the same | the rung *is* the resident kernel |
| **Constraint** | ensure as a penalty schedule or a projection; encourage as a quadratic term | the same; satisfaction read after integration | a kinematic write must skip pinned bodies (Anneal fix, Density skip, `FlowAdvect` skip) | needs a pin mask (P6b) |

### 7.2 Each combinator, with evidence

**Weighted sum.** Today every term sums at weight one, each calibrated alone. Charge was matched at contact: "at contact (a node diameter, 36) the `1/d` push matches `NodeExclusion`'s inverse-square one" (`physics_catalog.rs:62-65`). That is a reference-configuration normalization done once by hand (F5). LAMMPS's `hybrid/scaled` and (SGD)² are the external forms of the weighted sum.

The Density evidence shows what an unconverted sum does.

- **Density + `EdgeSpring`.** Spearman 0.77 alone fell to −0.54 with 316 overlaps (plan, "Assessed and ruled", 2026-10-02; the source log was not located in `Code/testing/mere/density/`).
- **Density + each overlay**, 900 ticks on the 200-node graph (`Code/testing/mere/density/probe-overlays.log`; the probe is `density_overlay_probe` on `1bf47cf9`). Density alone reads rank 0.60 with 4 overlaps.

  | Overlay added | Rank | Overlaps | Note |
  |---|---|---|---|
  | Hub pull | −0.33 | 510 | |
  | Centre | −0.21 | 410 | |
  | Tide | −0.26 | 409 | |
  | Skeleton | −0.21 | 362 | |
  | Depth | −0.04 | 230 | |
  | Grid | 0.32 | 80 | |
  | Hub room | 0.57 | 25 | density CV 7.32 |
  | Group pull | 0.60 | 4 | no change: every generated node is its own site |

The mechanism is recorded on the branch. The overlay "is a force rapier integrates as velocity in the same step Density writes translations" (`1bf47cf9` plan, Findings, P6a), so two currencies are summed at two points of the step.

*Reading, not ruled:* the JKO framework gives the ruled conversion a definite meaning for Density. A Fokker–Planck equation with a drift `−∇V` is the Wasserstein gradient flow of `∫ρ log ρ + ∫Vρ`, and the granular-media line extends this to pairwise interaction energies. Adding `−∇V/γ` to the advection velocity *before* Density writes the translation is the composition that has an objective: the cartogram's entropy plus the overlay's energy. Its equilibrium is no longer "area ∝ mass", so whether the composite carries an inference is a separate test. The ruled "Bonds … accepted only if rank stays at least 0.8" is that test.

**Scope and partition.** The kind matrix is a partition of pair space (LAMMPS `hybrid`). Couplings already carry a scope (numen `NodeSelector`: All, Tagged, Kind, NotTagged). Group pull carries one implicitly (site). A spec makes scope a first-class field on every term: a set predicate in SetCoLa's sense, evaluated against host sources and deferred to the runtime, so a saved spec reapplies to a second graph.

**Groups.** P7c reads "an outer law over group centroids …, each centroid's force spread over its members, and an inner law per group under a membership mask." Under OpenMM's centroid definition and the chain rule, an outer energy `U(c_g)` with `c_g = Σ w_i·x_i / Σ w_i` gives member `i` the force `(w_i/Σw)·F_g`. The group then responds as one body of the group's total mass, as an FM³ coarse node does. Giving each member the full `F_g` makes the group respond as one body of a single member's mass. That is stronger, and not a gradient (F9).

**Sequence and schedule.** The tree has four schedules already:

- Anneal's cooling (`anneal.rs:170`);
- Density's passes (branch plan, P6a second round: "one-second passes reach 0.81 at 3 600 ticks");
- the settle budget, a fixed 360 ticks;
- P5's staleness rule, "the newest completed result applies for up to N steps".

d3's alpha, t-SNE's early exaggeration (an attraction multiplier for the first iterations, which Böhm et al. read as a point on the spectrum) and OpenMM's MTS groups are the external forms. "Capture positions as anchors" (P7e) is the schedule step that turns a dynamics result into a target. It is how a dynamics composition hands a stable picture to an energy one.

**Level-of-detail condition.** The rung chosen for a term is a condition on measured facts, as in Gosling's visibility conditions: node count, device present, host, with hysteresis. P5 ruled the measure and the default ("Node count per host, measured", 0 meaning always GPU). The rung contract is already a type: `RepulsionRequest` exists "so it cannot quietly omit `NodeExclusion`'s cutoff or substitute smooth softening for its hard floor" (`seiche/lib.rs:262-265`). It is also already a receipt: P5a matched the CPU law to 2.9e-6 at 1k nodes (plan, P5 rulings).

**Constraint.** Three realizations of ensure are in the tree, none with a satisfaction report:

- kinematic pins (`seiche/lib.rs:829`);
- contacts (soft, read afterwards as overlaps);
- the skip of non-dynamic bodies in every state-writer.

One realization of encourage exists, the anchor spring, whose residual is not reported. `sceno` already reports `HonoredHold` and `unmet_holds` for scores (`sceno/score.rs:107-120`). The physics side has no counterpart.

### 7.3 Carried inference

Mark's lens is to judge a composition by the inference it carries, not by its resemblance to something. Under that lens a composition is meaningful when its signature observable still supports a reading that neither part supports alone (*Reading* on how the cases split):

- **Stress + Depth: "graph distance faithful, depth downward."** A carried inference, named in production: neato's `mode=hier`, "top-down directionality similar to dot".
- **Charge between meaning groups + Springs within (P7c's first instance): two inferences at two scales.** Distance between islands reads as semantic distance; shape within an island reads as structure. It is legible when the scales separate. P7's done-condition, a between-group gap over within-group spread with a shuffled-meaning control, tests exactly this.
- **Density + Depth: "area ∝ mass, stratified by depth"** (*Reading*). Under the free-energy sum the equilibrium is Boltzmann-weighted along the depth axis, a barometric picture. It is testable as rank within depth bands. Not tried.
- **Density + `EdgeSpring`: none.** Rank −0.54. Refused by ruling until a Bonds term clears 0.8.
- **Density + Hub pull: none.** Rank −0.33, 510 overlaps.
- **Kinds by site + Group pull by site: redundant.** The same channel twice is a weight, not a new inference.
- **Orbit + any dissipative overlay: destroys Orbit's inference.** `counter_damping` exists to protect it (`gravity.rs:36-40`).
- **Any law + Tide: an ambient inference** ("a living display"), not a structural one.
- **Stress, then capture, then Springs: "the metric map, relaxed locally, remembering it."** The mental-map case for sequenced blends.

## 8. Observables as receipts, and effectiveness beside the grammar

The observables exist:

- `LayoutStats` carries energy, spread, overlaps and stretch (`physics_catalog.rs:391-404`), and the branch adds `mass_area_rank` and `density_cv`;
- each law's P2 receipt asserts its signature;
- the P1 tests state "what it reveals" per law and overlay (plan, P1).

A grammar makes three of these structural (*Reading, not ruled*):

- **Each term declares its signature observable.** Stress declares stretch, Orbit an energy floor, Density rank.
- **A composition declares which signatures it must preserve, with bars.** The ruled Bonds bar (rank ≥ 0.8) is the first such declaration.
- **Effectiveness knowledge is kept as data beside the spec, versioned separately.** Each row records composition, fixture, observable, value, bar, date and commit. Draco 2 "replaced the knowledge base without changing what a spec means" (catalog line 427), and the same separation lets a probe add a row without a spec version bump. Its first rows are the P2 receipts, the P4 drag rows and the density probes above.

## 9. A portable dynamics spec

What it would contain (illustrative, not compile-ready):

```rust
// Illustrative shape only.
struct DynamicsSpec {
    version: u16,                         // DYNAMICS_SPEC_VERSION, as SCORE_VERSION
    terms: Vec<Term>,                     // energy and dynamics terms
    target: Option<TargetRef>,            // the arrangement (by Score / strategy id) and its satisfaction:
                                          //   Encourage { pull } or Ensure
    combinators: Vec<Combinator>,         // Groups { partition: Channel, outer, inner, spread },
                                          //   Schedule { stages, capture_as_anchor }
    realization: Realization,             // Integrate { damping, dt, budget } | Optimize | Anneal { seed, cooling }
    seed: u64,
    observables: Vec<Bar>,                // signatures to compute and the bars to preserve
}
struct Term {
    kind: TermKind,                       // the kernel family id (e.g. "repulsion.inverse-square", "kuramoto")
    scope: Scope,                         // All | Edges | Pairs(Channel) | Groups(Channel) | KindMatrix(Channel) | Selector
    params: Params,                       // kernel parameters, at the common scale
    weight: f32,
    sources: Vec<(Slot, Channel)>,        // mass <- degree | pagerank | meaning; kind <- site | cluster | ...
    rung: Option<RungId>,                 // pinned for reproducibility; None = host policy
}
// Derived, never authored: currency, class (E/Em/H/N/K), metric — from TermKind and sources.
```

**What it leaves out.** Positions belong to the score and scene: realized placements go in as `Placement::Coordinate` and holds, per "solver proposes, the score records" and A6's one serialization. Also left out:

- the device and the thresholds, which are host facts (P5 rulings);
- the effectiveness table, which lives beside the spec (§8);
- couplings, which are field-layer *authority* with identity, lifecycle and persistence (`numen/coupling.rs`, module doc). The spec references them; it does not copy them.

**Relation to the `Score` and the A6 seam.** The `Score` records *what* was chosen for an analytic arrangement. The dynamics spec records *how* a live layout was driven. Together they reproduce a scene: the arrangement's slots, the terms that act around them, and the holds that outrank both.

Determinism is partial today:

- laws iterate nodes in sorted order so seeded runs reproduce (`seiche/laws/mod.rs:65-83`);
- Anneal and Kinds are seeded (`LAW_SEED`, `physics_catalog.rs:56`);
- seiche's manifest names no rapier features (`crates/conatus/seiche/Cargo.toml`, line 19), so cross-platform bit-identity is not claimed. Whether a spec promises reproduction or only resemblance is a receipt question, unverified here.

**How it would travel.** `SavedSceneV1` already carries the choice in flattened form: law, overlays and three sources (`ports/graphshell/src/product.rs`, lines 216–243), plus damping, paused and `arrangement_pull`. Affinity is absent, as the plan noted. Shelfmark lets a target own an opaque delta section, which is how mer3ly carries `mer3ly.motion`. So a citation could carry a `dynamics` section with no envelope change (*Reading*).

**Its forcing consumers.** The first is Graphshell's own saved scene. The remote board mirrors the canvas's choice every frame (plan, P3). mer3ly is a candidate second consumer only for its motion class (F-h).

## 10. Algorithmic diversity: rungs per term

| Term | Rungs in the tree | Rungs in the literature | Selected by |
|---|---|---|---|
| Inverse-square repulsion with cutoff (`NodeExclusion`) | CPU exact (`seiche/forces.rs:65-135`); staged closure (`seiche/lib.rs:364`); Burn `[N, N]` tensors (`seiche/tensor_forces.rs`); CubeCL tiled (`438187cb`); CubeCL cell list (`7784ddf5`); lagged seam (`03cb4207`); resident `integrate` (`crates/conatus/conatus/src/resident/kernels.rs`, line 177) | Verlet neighbour lists | node count per host, measured (P5) |
| `1/d` charge (Charge, Hub pull) | Barnes–Hut on the CPU (`seiche/barnes_hut.rs:57`) | FM³ multipole; GPU Barnes–Hut; PME-style mesh | none today |
| Stress | integration of pairwise springs (`stress.rs`) | KK Newton; majorization; SGD; sparse pivots | none today |
| Spring-electrical energy | integration (Springs); Metropolis (Anneal) | L-BFGS (OpenMM's minimizer, Penrose) | the law id |
| Hall's energy | closed form by power iteration (`producers.rs:139`) | Lanczos; multilevel eigensolvers | the arrangement id |
| Density | CPU grid with Jacobi sweeps (branch); GPU grid (P6b, planned) | the 2018 flow-based algorithm | the CPU tier's existence (ruled "Ordinary laws with a CPU tier") |
| Phase coupling | explicit Euler (`kuramoto.rs:103`) | — | — |

How a spec selects (*Reading, not ruled*). The spec names terms and a realization *policy*: integrate, optimize, anneal or closed form. It may pin a rung for a receipt. Otherwise host policy picks rungs inside that policy from measured conditions, as P5 ruled, and every rung must reproduce its term's law to a stated tolerance, as P5a's parity receipt does. Two rules follow from §6.

- **Kernel range decides the rung family.** Cutoff kernels admit cell lists; long-range kernels admit Barnes–Hut or multipole methods. This is OpenMM's `NonbondedForce` method list, which is why the cell list fits `NodeExclusion` and not Charge.
- **Class decides the realization family.** Optimizer rungs exist only for energy compositions under one metric.

## 11. Forks for Mark

Each fork gives the evidence, then two to four options, recommendation first.

**F1. The model's shape (the hypothesis's result).**
Evidence: 6 of 12 laws, 5 of 8 overlays, the anchor and affinity slots, and 1 of 8 listed arrangements fit an objective model. Five laws are dynamics-only or have a misleading minimizer, and four arrangements are generators (§6.2). The anchor spring already turns every arrangement into a target term.
1. **Terms and targets** (recommended). One specification model with energy, dynamics and target terms. Arrangements stay `Score`-recorded generators referenced as targets. Optimizer realizations are gated on class and metric. This commits the grammar plan to declaring a class per term and keeps the seam doc's split.
2. **Energy-only grammar.** The grammar covers the energy class and targets. Orbit, Sync, Kinds and Flock stay a separate "living laws" catalog outside it. This is simpler, but composing a living law with anything stays ad hoc.
3. **Separate grammars sharing sources.** Arrangements and laws stay separate catalogs and share only one channel registry. This is the smallest change, and it gives up optimizer realizations and twins.

**F2. Where P7 lives, and how it is rewritten.**
Evidence: P7a–e map onto the grammar one to one. P7a is term declarations, P7b a channel, P7c the Groups combinator, P7d a weighted sum at a common scale, P7e the Schedule combinator. The grammar adds the class instrument, satisfaction reports and the spec artifact, none of which P7 had. Mark ruled the grammar plan "Its own doc beside projection grammar"; P7's own home was not ruled.
1. **P7 moves into the grammar plan as its implementation tracks** (recommended). The physics plan's P7 closes with a pointer, and P5 and P6 stay where they are. One plan then owns composition, and the physics plan stays the catalog's.
2. **P7 stays in the physics plan, rewritten in grammar terms.** The grammar plan holds only the model and the spec. That leaves two documents describing composition.
3. **P7 proceeds as written, and the grammar retrofits it.** The fastest path to a grouped layout, but the classes and the common scale arrive after the combinators that need them.

Draft rewrite under option 1 (illustrative wording, for the grammar plan):
- *G1, terms declared.* Every law, overlay and slot declares topology, kernel family, state, currency, class, metric channel and signature observable, matching §5's tables. Done when an energy-descent test and a reciprocity test agree with every declared class, and a deliberately non-conservative term (Kinds with its seeded matrix) fails the descent test (positive control).
- *G2, one channel registry.* Kind, mass, depth, groups, pairs and distances become channels shared with cartography's disclosures. Meaning joins as a channel (P7b's ruling unchanged), and Group pull takes any group channel (F-g). Done when Columns-by-cluster has a law-form twin and Meaning feeds Kinds, Group pull, Affinity and Groups from one snapshot.
- *G3, combinators.* A weighted sum at the common scale (F5), with P7d's 1/0 and 0/1 check. Groups with the chosen spread rule (F9), with P7c's done-condition unchanged. A schedule with capture-as-anchor, with P7e's tolerance check. Currencies as ruled ("Laws declare currency; catalog adapts or refuses"), with Density's adapt branch defined as the free-energy sum (§7.2).
- *G4, the spec.* `DynamicsSpec` in the saved scene, reopened byte-for-byte. Unknown term kinds fail explicitly, as Graphshell's compiler does for unsupported registrations.
- *G5, realizations.* One optimizer rung for energy compositions, checked against integration on the same fixture.
- *G6, satisfaction.* Pins honored, anchor residual RMS and contact overlaps reported per composition.

**F3. The grammar plan's first tracks.**
Evidence: the derived classes (§6) are the riskiest claims in this brief, and they are cheap to measure. The spec artifact depends on them. P7's combinators depend on the common scale.
1. **Declarations and instruments first** (recommended): G1, then G2, G3, G4, G5, G6 as drafted under F2. This front-loads measurement and behaviour change comes second.
2. **The artifact first.** Define `DynamicsSpec` and its saved-scene field, then fill it. Visible sooner, but the type may freeze before the classes are measured.
3. **The combinators first**, P7 as written, with metadata retrofitted afterwards.

**F4. Where the spec lives.**
Evidence: the projection grammar's rule is portable "only when a named consumer forces it" (catalog lines 357–360). Mark's 2026-09-23 principle is "Design for known futures now; implement in order" (DOC_README, Working principles). Today's consumers are Graphshell's saved scene and the remote board. mer3ly carries only a motion class (F-h). seiche is the published home of laws and overlays (ruled 2026-10-01, "Law in seiche, kernels in conatus") and is kernel-free.
1. **Design the portable shape now, in seiche** (recommended). Sources resolve host-side in the canvas, and `PhysicsChoice` becomes the binding. It is carried in `SavedSceneV1` now, and as a shelfmark delta section when a citing consumer asks. This commits seiche to a serialized type.
2. **Grow `PhysicsChoice` in the canvas.** Portable only when a second consumer forces it. This follows the projection grammar's promotion rule to the letter.
3. **In `sceno`, beside the `Score`** (Score v5 or a sibling). That puts physics vocabulary into the product-free scene crate.

**F5. The common strength scale (P7d's prerequisite).**
Evidence: strengths are calibrated alone, across four orders of magnitude: 220 000, 6 000, 60 000, 9 000, 2 500, 4 000, 40 000, 2 000, plus Anneal's energy constant of 500 000. The one cross-term calibration was by hand at contact (`physics_catalog.rs:62-65`).
1. **Reference-configuration normalization** (recommended). A term's weight-1 strength is defined by its force at a declared reference: contact distance (36) for repulsions, as Charge's precedent did; one rest length of stretch for springs; one rest length of offset for unary pulls. It is cheap, local and receipt-checkable.
2. **Energy normalization.** Each term's energy at a fixture's Springs equilibrium is normalized to 1. Meaningful for energy terms only.
3. **Per-pair empirical calibration** against receipts, as today. It does not scale to weighted lists.

**F6. Anneal's place.**
Evidence: Anneal's energy is Springs' functional form with other constants, and it lacks Davidson–Harel's crossing term (F-b). The 2026-10-01 ruling: "Don't present alt tunings as alt instruments", and new law ids are "for novel dynamics only".
1. **Annealing becomes a realization** any energy composition can select ("settle by annealing") (recommended). The id `anneal.davidson-harel` keeps opening, as Springs' terms under annealing, so saved scenes reopen. This removes one law from the picker and adds a realization control.
2. **Keep Anneal a law, and give it what only annealing can carry.** Its energy becomes Springs' term set by construction, plus an edge-crossing term. It would then reveal something integration cannot.
3. **Leave it as is.**

**F7. Energy's kernel.**
Evidence: the code is ForceAtlas2's (1, −1); the id and the doc say LinLog's (0, −1), which is the exponent Noack's modularity result attaches to (F-a). The receipts are calibrated against (1, −1).
1. **Keep (1, −1) and correct the docs and label to ForceAtlas2** (recommended). The attraction exponent becomes a kernel parameter, so LinLog proper is a tuning, which the 2026-10-01 ruling welcomes. The id stays for saved scenes.
2. **Switch the kernel to true LinLog** under the same id. Pictures change, and the Energy receipt and P1 test are re-run.
3. **Both as separate laws.** Against the 2026-10-01 ruling, since they are two tunings of one kernel.

**F8. Terms that are non-conservative by accident.**
Evidence: Hub room and Hub pull are gradients only in their own mass metric (F-c). Flow's needle has a curl because of a length factor (F-d). Neither looks intended. Both block optimizer realizations for any composition that contains them.
1. **Declare them as they are (Em, N)** and let the class gate realizations (recommended). No picture changes now. Revisit when G5 builds an optimizer rung.
2. **Make them gradients now.** Hub weights become reciprocal (`w_i·w_j`), and the needle drops its length factor. Pictures change, and the overlay tests and the Flow receipt are re-run.
3. **Offer both forms as a kernel parameter.**

**F9. The grouped spread rule (P7c).**
Evidence: P7c says "each centroid's force spread over its members" without a rule. The chain rule through a weighted centroid, OpenMM's `CustomCentroidBondForce` definition, gives each member its weight share, and the group responds with its total mass, like an FM³ coarse node. A full force per member makes the group respond as a single member's mass.
1. **The weight share** (recommended). This is the gradient: it composes with energy terms and optimizers, and the outer law's strength does not grow with group size.
2. **The full force per member.** Stronger separation between groups at the same weight, but not a gradient, so the grouped composition becomes dynamics-only.

**F10. Whether Kinds keeps ticking.**
Evidence: the plan's finding says Kinds never rests; `never_rests` excludes it (F-f). The P2 receipt reads its energy only at 1 s, so the exclusion may be deliberate (it settles) or an omission.
1. **Measure first** (recommended): Kinds' energy at 6 s and 30 s under continuous ticking on the P2 fixture. Add it to `never_rests` if the energy stays above the floor; otherwise correct the plan's finding.
2. **Add it to `never_rests` now.**
3. **Leave it, and correct the plan's finding.**

## 12. Sources

Fetched or searched 2026-10-02.

- OpenMM 8.6 custom forces: <https://docs.openmm.org/latest/userguide/theory/03_custom_forces.html>; standard forces: <https://docs.openmm.org/latest/userguide/theory/02_standard_forces.html>; integrators: <https://docs.openmm.org/latest/userguide/theory/04_integrators.html>; `LocalEnergyMinimizer`: <https://docs.openmm.org/latest/api-c++/generated/LocalEnergyMinimizer.html>; `MTSLangevinIntegrator`: <https://docs.openmm.org/latest/api-python/generated/openmm.mtsintegrator.MTSLangevinIntegrator.html>
- LAMMPS (release 30Sep2026) `pair_style hybrid`: <https://docs.lammps.org/pair_hybrid.html>
- HOOMD-blue integration methods (latest 7.1.2): <https://hoomd-blue.readthedocs.io/en/v5.2.0/hoomd/md/methods/brownian.html>
- d3-force 3.0.0: <https://d3js.org/d3-force/simulation>, <https://d3js.org/d3-force/many-body>, <https://d3js.org/d3-force/link>
- XPBD: <https://dl.acm.org/doi/10.1145/2994258.2994272>
- Box2D Solver2D: <https://box2d.org/posts/2024/02/solver2d/>
- Lenia: <https://www.complex-systems.com/abstracts/v28_i03_a01/>; Particle Lenia: <https://google-research.github.io/self-organising-systems/particle-lenia/>
- Non-reciprocal phase transitions: <https://www.nature.com/articles/s41586-021-03375-9>
- Kuramoto Lyapunov function: <https://link.springer.com/article/10.1007/BF01048044>; Kuramoto–Sakaguchi is not a Wasserstein gradient flow: <https://arxiv.org/abs/1908.07657>
- Fruchterman–Reingold: <https://www.reingold.co/force-directed.pdf>
- Stress by SGD: <https://doi.org/10.1109/tvcg.2018.2859997>; stress majorization (Gansner, Koren, North, GD 2004): <https://www.semanticscholar.org/paper/Graph-Drawing-by-Stress-Majorization-Gansner-Koren/c320716ff3a885ab081d2cf8cbc520c50e046dc2>
- Graphviz 16.1.0 neato `mode`: <https://graphviz.org/docs/attrs/mode/>; sfdp: <https://graphviz.org/docs/layouts/sfdp/>
- OGDF 2023.09 `ModularMultilevelMixer`: <https://ogdf.github.io/doc/ogdf/classogdf_1_1_modular_multilevel_mixer.html>; FM³ (GD 2004): <https://www.researchgate.net/publication/30508821_Drawing_Large_Graphs_with_a_Potential-Field-Based_Multilevel_Algorithm>
- ForceAtlas2: <https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0098679>; Noack 2009: <https://doi.org/10.1103/PhysRevE.79.026102>
- WebCoLa: <https://ialab.it.monash.edu/webcola/>; IPSep-CoLa: <https://research.monash.edu/en/publications/ipsep-cola-an-incremental-procedure-for-separation-constraint-lay/>; SetCoLa: <https://github.com/uwdata/setcola>
- Penrose: <https://penrose.cs.cmu.edu/media/Penrose_SIGGRAPH2020a.pdf>
- Davidson–Harel: <https://dl.acm.org/doi/10.1145/234535.234538>
- Magnetic spring model: <https://www.semanticscholar.org/paper/Graph-Drawing-by-the-Magnetic-Spring-Model-Sugiyama-Misue/d6762e1874f8926a4ee18d58cf0b04cffce7451b>
- Koren, spectral: <https://doi.org/10.1016/j.camwa.2004.08.015>
- (SGD)²: <https://arxiv.org/abs/2112.01571>
- Attraction–repulsion spectrum: <https://arxiv.org/abs/2007.08902>
- Gastner–Newman: <https://www.pnas.org/doi/abs/10.1073/pnas.0400280101>; Gastner–Seguy–More: <https://doi.org/10.1073/pnas.1712674115>
- Energy-based learning: <http://yann.lecun.com/exdb/publis/pdf/lecun-06.pdf>
- JKO 1998: <https://www.researchgate.net/publication/2241596_The_Variational_Formulation_of_the_Fokker--Planck_Equation>; Carrillo–McCann–Villani: <https://ems.press/journals/rmi/articles/5018>

Cited from the reference record without a fetch this session: Kamada–Kawai (*IPL* 31(1), 1989), Hall (1970), Reynolds (1987), Hu (2005). Their bibliographic details are standard; the claims made of them are the ones their successors above restate.

**Not verified here:** the log behind the plan's "0.77 alone … −0.54" Density + `EdgeSpring` figure (the plan is the citation); rapier's cross-platform determinism; HOOMD's rule, if any, on a particle belonging to two methods. *Annotated 2026-10-02:* the physics catalog plan now records that the figure's first-round log was overwritten by the Density lane's second-round runs. `probe-overlays.log` is the surviving evidence of the same failure class.
