# Dynamics Grammar Plan

**Date:** 2026-10-02
**Status (2026-10-02):** planned. Written from the [dynamics grammar brief](../research/2026-10-02_dynamics_grammar_brief.md) and Mark's rulings of 2026-10-02; no track started. G1 comes first (ruled).
**Scope:** turn the physics catalog's laws, overlays and slots into one specification model of *terms and targets*. Every term declares what it is, and two instruments check each declaration. Sources become one channel registry. Composition gets declared semantics (a weighted sum at a common scale, groups, schedules, currencies). Choices travel as a portable `DynamicsSpec`. Annealing and one optimizer join integration as realizations, gated on each term's class. Pins, anchors and contacts report whether they were satisfied. This plan is P7 of the physics catalog plan, moved here by ruling F2.

Not in scope:
- P5 (repulsion on the host's GPU) and P6 (Density), which stay in the physics catalog plan. This plan consumes them as rungs and as a law.
- New laws or arrangements.
- 3D.
- Any change to `sceno::Score`.

**Related:**
- [dynamics grammar brief](../research/2026-10-02_dynamics_grammar_brief.md): the evidence, the decomposition tables (§5) and the hypothesis test (§6) this plan builds on.
- [physics catalog plan](2026-09-02_physics_catalog_plan.md): the catalog, P5 and P6, and the record of every ruling below.
- [projection grammar adoption plan](2026-08-15_projection_grammar_adoption_plan.md): the sibling this plan stands beside, and its discipline, "Solver proposes, the score records".
- [projection grammar catalog](../research/2026-08-15_projection_grammar_catalog.md): the grammar this one parallels, and its promotion rules.
- [the cartography–gyre layout seam](../technical_architecture/2026-05-29_cartography_aether_layout_seam.md): arrangements compute, physics simulates. Under F1 that split stands: arrangements produce targets, terms are realized.

*Reading, not ruled* marks this plan's own inference. Everything else is a ruling, a quotation, or a file and line.

## 1. Rulings

Mark's words are quoted as the physics catalog plan records them (§3, P7, and §5); that plan is the primary record. Each fork's question and evidence are in the brief's §11. The "Options" lines list the recommendation first, as the forks were put.

### 1.1 The grammar's own rulings (2026-10-02)

**Held for a grammar.** Mark: "in the way we've managed to describe projection grammar, we should probably think about an overarching model of physics; combinatorial, algorithmically diverse... more thoughts?"
- **Order.** "Research brief first, P7 held". Alternatives: the brief alongside P7a, or extending P7 directly. *Follows:* the brief, written 2026-10-02.
- **This plan's home.** "Its own doc beside projection grammar". Alternatives: inside the physics catalog plan, or inside the projection grammar plan. *Follows:* this document.
- **The hypothesis.** "Yes, as a hypothesis the brief tests". Alternatives: adopting it now, or keeping them separate. *Follows:* the brief found that the hypothesis holds for the energy class only (6 of 12 laws, 5 of 8 overlays, the anchor and affinity slots, Spectral alone among the arrangements). It fails for dynamics-only laws (Kinds, Flock, Flow's needle; Orbit and Sync, whose minimizers are collapse and total synchrony) and for generator arrangements.

**F1, the model.** Question: what shape the model takes, given the hypothesis's partial result. Options: terms and targets; an energy-only grammar with the living laws outside it; separate grammars sharing only sources. Mark: **"Terms and targets"**. *Follows:*
- One specification model of energy, dynamics and target terms.
- Arrangements stay `Score`-recorded generators, referenced as targets.
- Optimizer realizations are gated on class and metric.

**F2, P7's home.** Question: where P7 lives, and how it is rewritten. Options: P7 moves into the grammar plan; P7 rewritten in the physics plan; P7 as written. Mark: **"P7 moves into the grammar plan"**. *Follows:*
- P7 becomes this plan's tracks, G1 to G6.
- The physics catalog plan's P7 closes with a pointer here; P5 and P6 stay there.
- P7's done-conditions are carried into the tracks: currency pairs and composition in G3, Meaning in G2, save and reopen in G4.

**F3, first tracks.** Question: which track comes first. Options: declarations and instruments first; the spec artifact first; the combinators first. Mark: **"Declarations and instruments first"**. *Follows:*
- G1 comes first. Every term declares topology, kernel, state, currency, class and observable.
- An energy-descent test and a reciprocity test must agree with each declared class, and Kinds must fail descent as the positive control.
- Then G2 to G6.

**F4, the spec's home.** Question: where the portable spec lives. Options: portable shape now, in seiche; grow `PhysicsChoice` in the canvas; place it in sceno beside the `Score`. Mark: **"Portable shape now, in seiche"**. *Follows:*
- A `DynamicsSpec` in seiche, with sources resolved host-side and `PhysicsChoice` as the binding.
- It is carried in `SavedSceneV1` now, and as a shelfmark delta section when a citing consumer asks.

**F5, the common scale.** Question: how strengths are brought to one scale, the prerequisite for weighted lists. Options: reference-configuration normalization; energy normalization; per-pair calibration. Mark: **"Reference-configuration normalization"**. *Follows:* a term's weight-1 strength is its force at a declared reference:
- contact distance 36 for repulsions, as Charge's calibration did;
- one rest length of stretch for springs;
- one rest length of offset for unary pulls.

**F6, Anneal.** Question: Anneal's energy has Springs' functional form, so is it a law or a realization? Options: annealing becomes a realization; keep it a law with an edge-crossing term; leave it. Mark: **"Annealing becomes a realization"**. *Follows:*
- Any energy composition may settle by annealing.
- `anneal.davidson-harel` keeps opening, as Springs' terms under annealing, so saved scenes reopen.

**F7, Energy's kernel.** Question: the code runs ForceAtlas2's (1, −1) force model under a LinLog id and doc. Options: keep (1, −1) and relabel ForceAtlas2; switch to true LinLog; both as laws. Mark: **"Keep (1, −1), relabel ForceAtlas2"**. *Follows:*
- The code's ForceAtlas2 model stays, and its docs and label are corrected.
- The attraction exponent becomes a kernel parameter, so true LinLog is a tuning.
- The id `energy.linlog` stays.

**F8, accidental non-conservatism.** Question: what to do with Hub room, Hub pull and Flow's needle, which are non-conservative by accident. Options: declare them as they are; make them gradients now; offer both forms. Mark: **"Declare them as they are"**. *Follows:*
- They are classed honestly, and the class gates realizations.
- This is revisited when an optimizer rung is built (G5).

**F9, the grouped spread rule.** Question: how a group centroid's force reaches the members. Options: weight share; full force per member. Mark: **"Weight share"**. *Follows:* each member takes its weight share of its group's centroid force, which is the true gradient.

**F10, Kinds.** Question: `never_rests` excludes Kinds, against the physics plan's finding. Options: measure first; add it now; correct the finding now. Mark: **"Measure first"**. *Follows:* Kinds' energy at 6 s and 30 s under continuous ticking on the P2 fixture decides whether it joins `never_rests` or the plan's finding is corrected (G1).

### 1.2 Earlier rulings that are this plan's inputs

From the physics catalog plan, §3 P5–P7 and §5:

| Ruling | Mark's words | What it binds here |
|---|---|---|
| Currencies (2026-10-02) | "Laws declare currency; catalog adapts or refuses" | Forces compose freely. A kinematic law takes forces converted to its currency (overdamped, `v = F/γ`). A resident law takes only forces with resident kernels or the lagged upload. The picker greys out the rest with the reason. Declared in G1, enforced in G3. |
| Composition tier (2026-10-02) | "Grouping overlays: semantic + partitioned"; "Weighted multi-law lists"; "Sequenced blends" | A `Grouped` outer law on centroids with an inner law per group, plus a semantic overlay, each with a source and weight. `law` becomes a weighted list at a common scale. Profiles carry a schedule, with capture-as-anchor. All in G3. |
| Meaning (2026-10-02) | "'Meaning' as a source" | Beside site and cluster, feeding affinity pairs, `DomainCluster` groups, Kinds' kinds and the partition for grouped laws (G2). |
| Embeddings (2026-10-02) | "Burn on the host device, off-path" | ESP's `bert::load_wgpu` is fixed to take the host's device. Embeddings are computed on content change in an actor or async task. Lexical embeddings on the CPU are the fallback, and the wasm default until ESP's wasm build is verified (G2). |
| Semantic field (2026-10-02) | "Snapshots only for now" | Meaning enters as pairs, groups and kinds snapshots through the existing rebuild path; no per-step semantic field (G2). |
| Density with edges (2026-10-02) | "Pure Density now, Bonds later" | Edges may join Density as a "Bonds" overlay in Density's currency, accepted only if rank stays at least 0.8 (G3). |
| Overlays on Density (2026-10-02) | "Refuse overlays on Density now" | The catalog refuses overlays on Density with a reason "until the currency work", which is G3. |
| What earns a law id (2026-10-01) | "I don't mind tunings. Don't present alt tunings as alt instruments." | New law ids are for novel dynamics only. Scale versions stay under the law's id as a backend tier. This underlies F6 and F7. |
| Home of GPU-tier laws (2026-10-01) | "Law in seiche, kernels in conatus" | Terms are declared in seiche. Resident rungs live in `conatus::resident`. |
| The picker (2026-10-01) | "Ordinary laws with a CPU tier" | Every term keeps a CPU rung, so every host can realize every spec. |
| Rapier's role (2026-10-01) | "Rapier seems the fallback in any case" | Integration under rapier remains the default realization, and every other realization falls back to it. |
| Licensing (2026-10-01) | "Doublecheck. Otherwise, 1" | Nexus-derived code is treated as Apache-2.0. A close port gets a `LICENSES.md` entry; code merely informed by it stays MPL-2.0 with a credit line. |

**Rules for lanes on this plan** (*Reading, not ruled*, gathered from current policy):
- **Licensing.** The licence posture brief's ruling is "MPL-2.0 by default, with correct provenance", and "there are no exceptions" (`design_docs/2026-08-22_license_posture_brief.md`). Algorithms are written from the literature where possible, as P6a's CPU tier was. Any close port gets a `LICENSES.md` entry under the licensing ruling above.
- **Instruments.** Every negative receipt carries a positive control in the same run.
- **Forks.** A lane stops at any choice with more than one defensible answer and returns it as a fork. The open questions in §3 are those already visible.

## 2. Tracks

G1 runs first (F3). After it, *Reading, not ruled*: G2 and G6 depend only on G1; G3 needs G1 and, for the Meaning instance, G2; G4 needs G1 and G3's combinator shapes; G5 needs G1's energies and G4's spec. Where two tracks could run side by side, the order goes back to Mark.

### G1 — declarations and instruments

Every term declares what it is, and two instruments check the declarations. This is P7a's "every law declares its currency", widened to the whole declaration.

- **Declarations.** Every law term, overlay, slot and always-on term (the rows of the brief's §5.1–5.3) declares:
  - its topology: unary, edges, pair list, all-pairs with an optional cutoff, groups, kind matrix, or medium;
  - its kernel family;
  - its state moved;
  - its currency;
  - its class: E, Em, H, N or K, as the brief defines them;
  - its metric channel, for Em terms;
  - its signature observable.
  
  An E or Em term also exposes its energy function, so G5 can optimize it. *Reading, not ruled:* the declarations sit on the seiche term types, since F4 puts the spec in seiche. The canvas catalogs read them rather than restating them.
- **The descent instrument.** Under overdamped flow, an energy-declaring term's energy must never rise beyond tolerance, from seeded starts on the P2 fixture and on one generated graph. Em terms are run in their declared metric.
- **The reciprocity instrument.** For a pair term, the metric-weighted force sum `Σ m_i F_i` must be zero, with `m = 1` for E terms and the declared weights for Em terms. Unary terms are excluded as external.
- **The positive control.** Kinds, with its seeded asymmetric matrix, must fail descent and reciprocity. The same harness with a symmetrized matrix must pass both, which proves the instrument can say yes as well as no.
- **F7's relabel.** `LinLogForce`'s doc (`crates/conatus/seiche/src/laws/linlog.rs`, lines 7–17) and `PhysicsLaw::Energy`'s doc (`crates/canvas/pictograph/src/canvas/physics_catalog.rs`, lines 81–83) say ForceAtlas2's (1, −1) model. The attraction exponent becomes a kernel parameter: 1 by default, 0 for LinLog proper. `degree_weighted: false` is no longer described as "LinLog's reading". The id `energy.linlog` stays.
- **F10's measurement.** Kinds' energy at 6 s and 30 s under continuous ticking on the P2 fixture is recorded here. If it stays above the floor, Kinds joins `PhysicsLaw::never_rests` (`physics_catalog.rs`, lines 160–165). Otherwise the physics catalog plan's 2026-09-02 finding is corrected with a dated note.

*Done when:*
- every term in the brief's §5.1–5.3 has a declaration, and a catalog test proves none is missing;
- the descent and reciprocity instruments agree with every declared class, including Hub room and Hub pull as Em and Flow's needle as N (F8);
- Kinds' seeded matrix fails both instruments and the symmetrized matrix passes both, in the same run;
- the Energy law's P1 test and P2 receipt are unchanged at the default exponent, and exponent 0 runs (true LinLog as a tuning);
- F10's two figures are recorded, with the resulting change applied;
- the eleven law receipts stay green.

### G2 — one channel registry, Meaning included

Sources become one registry of channels, which laws and arrangements both read. This is P7b, plus the brief's finding F-g.

- **Channels.** Kind (site, cluster, colouring, island, degree, meaning), mass (degree, PageRank), depth (roots, layers, focus), groups (any kind channel), pairs (structural, content, blend), and distances.
  - The kind, mass and depth channels are `LawSources` today (`physics_catalog.rs`, line 599).
  - The pairs channel is the affinity signal and its blend (`crates/canvas/pictograph/src/canvas/strategy.rs`, lines 520–616).
  - Group pull takes any group channel, no longer site alone (the brief's finding F-g; `physics_catalog.rs`, line 1087).
- **Where it lives.** F4 says sources resolve host-side, so the registry lives in the canvas, and seiche's spec names channels by id. *Reading, not ruled:* the registry also produces cartography's disclosures (categorical and numeric axis, weight, embedding), so Columns-by-cluster and Kinds-by-cluster read one computation.
- **Meaning.** ESP embeddings run on the host device, off-path, recomputed on content change. `load_wgpu` is fixed to take the host's device. Lexical embeddings on the CPU are the fallback and the wasm default. One snapshot yields top-k pairs (`affinity_pairs_over_index`, `crates/intel/esp/src/embed/index_burn.rs`, line 144), cluster assignments and kinds. It feeds Affinity, which gives `set_content_affinity` its first non-test caller, plus Group pull, Kinds and G3's Groups partition.

*Done when:*
- Group pull by cluster exists, giving "Columns (by cluster)" a law-form twin, with a with/without test;
- one Meaning snapshot feeds Affinity, Group pull, Kinds and Groups, with one embedding run per content revision asserted;
- P7's Meaning condition holds: on a fixture graph with known topics, cluster purity against the topics is recorded on native GPU and on the CPU fallback, and the GPU path shares the host's device (a single device asserted);
- the wasm build takes the lexical fallback;
- the eleven law receipts stay green.

### G3 — combinators and currencies

The combinators get declared semantics, and currencies are enforced. This is P7a's enforcement plus P7c, P7d and P7e.

- **Weighted sum at the common scale (F5).** Each term's weight-1 strength is its force at its declared reference. Repulsions use contact distance 36, Charge's calibration (`physics_catalog.rs`, lines 62–65). Springs use one rest length of stretch, and unary pulls one rest length of offset. Today's calibrated strengths are re-expressed as weights, so the default compositions reproduce today's forces (*Reading, not ruled*: that compatibility requirement). `law` becomes a weighted list, mixed within one currency.
- **Groups (F9).** An outer law over group centroids, each member taking its weight share of its group's centroid force, plus an inner law per group under a membership mask. The first instance is the one Mark named: Charge (Barnes–Hut) between meaning clusters, springs within.
- **Schedule.** A profile may carry a schedule of compositions with stage stop conditions. Density's ruled stop test, "Shift < 0.05 for 3 passes", is the precedent for a stop condition. "Capture positions as anchors" freezes one stage's layout as the next stage's target, through the anchor slot.
- **Currencies.** Forces compose freely. A kinematic law takes forces converted to `v = F/γ` before its own write. A resident law takes only forces with resident kernels or the lagged upload. Everything else is refused, and both pickers grey it out with the reason.
- **Density.** The interim "Refuse overlays on Density now" builds on the refusal seam already on `density-cpu` (`PhysicsLaw::overlay_refusal`, `5f529c64`). Once the conversion lands, the brief's §7.2 *Reading* is that converting a force into Density's velocity is the free-energy sum. Which overlays Density then admits, and at what bars (Bonds' rank ≥ 0.8 is the ruled one), goes back to Mark with measurements.

*Done when* (P7's conditions, carried):
- every law reports its currency, and the pickers refuse an incompatible mix with its reason, with a test per currency pair;
- each term at weight 1 produces unit force at its reference within 1e-3 (the scale receipt);
- a weighted mix of two force laws matches each pure law at weights 1/0 and 0/1;
- the grouped Charge-between, springs-within layout separates meaning clusters: the between-cluster gap exceeds the within-cluster spread, stated as a ratio. It keeps edge structure inside each cluster, with a within-cluster stress figure no worse than Springs alone. Springs alone is the negative control, and a shuffled-meaning control fails the separation;
- the grouped composition built from energy terms passes G1's descent and reciprocity instruments, and the full-force variant (F9's rejected option) fails reciprocity on groups of unequal size (positive control);
- a sequenced blend reproduces the captured anchor layout within a stated tolerance;
- the eleven law receipts and Density's stay green.

### G4 — the `DynamicsSpec`

The portable artifact lands in seiche (F4).

- **The type.** `DynamicsSpec` in seiche: a version, terms (kind, scope, parameters at the common scale, weight, channel ids, an optional pinned rung), a target reference (arrangement plus satisfaction class), combinators, realization, seed, and observables with bars. This is the shape of the brief's §9, which is illustrative.
- **Derived fields.** Currency, class and metric are derived from G1's declarations, never authored.
- **The binding.** `PhysicsChoice` (`crates/canvas/pictograph/src/canvas/physics_board.rs`, line 48) becomes the binding: host-side channel resolution, plus the affinity toggle P7a named.
- **The carrier.** `SavedSceneV1` (`ports/graphshell/src/product.rs`, lines 216–243) carries the spec, with a serde default so a legacy scene opens as it does today. The remote board mirrors the canvas's spec, as it mirrors the choice today (physics catalog plan, P3).
- **Not built now.** The shelfmark delta section, until a citing consumer asks (F4). mer3ly's citation carries no physics today (brief, finding F-h).

*Done when:*
- every current choice (law × overlays × the three sources, and every profile) maps to a spec and back without loss;
- a spec saves and reopens byte-for-byte with the scene, and compositions save and reopen (P7's condition);
- a legacy `SavedSceneV1` with no spec opens identically;
- an unknown term kind or channel fails explicitly, never silently substituted, as Graphshell's projection compiler does;
- the remote board's receipt (`physics_remote_board.scn`) stays green;
- the eleven law receipts stay green.

### G5 — realizations

Integration, annealing and one optimizer become realization choices, gated on class (F1, F6, F8).

- **Integration.** Rapier stays the default and the fallback.
- **Annealing (F6).** Annealing becomes a realization of any composition whose terms are all E under one metric: Metropolis moves over the composition's declared energy (G1), with the cooling schedule as a realization parameter. `anneal.davidson-harel` reopens as Springs' terms under annealing, and its private energy copy (`crates/conatus/seiche/src/laws/anneal.rs`, lines 66–91) is retired in favour of the declared one (*Reading, not ruled*: the retirement).
- **One optimizer rung.** It is gated the same way, and is refused with the reason for any composition containing an H, N, K or mixed-metric term.
- **Rungs.** P5's repulsion rungs and Density's tiers are recorded as rungs of their terms (no rework). A rung must reproduce its term's law to a stated tolerance.
- **F8's revisit.** When the optimizer rung exists, the figures for making Hub room, Hub pull and Flow's needle gradients go back to Mark.

*Done when:*
- annealing is selectable for every all-E composition and refused with its reason otherwise, with a test per class;
- `anneal.davidson-harel` reopens from a saved scene and its P2 receipt (energy ≤ 5) stays green;
- on the P2 fixture, the optimizer rung reaches an energy no higher than integration's (within a stated tolerance) for Springs, Stress and Energy, and each law's signature receipt passes on its result;
- compositions with Orbit, Sync, Kinds, Flock, Flow or a hub overlay mixed with identity-metric terms are refused, with their reasons;
- F8's figures are in front of Mark;
- the eleven law receipts stay green.

### G6 — satisfaction reports

Constraints report whether they were satisfied, the physics counterpart of `sceno`'s honored and unmet holds (`crates/cambium/scenes/sceno/src/score.rs`, lines 107–120).

- **Pins** (ensure-class, kinematic bodies, `crates/conatus/seiche/src/lib.rs`, lines 829–838) report honored or unmet against a tolerance.
- **Anchors** (encourage-class) report their residual RMS.
- **Contacts** keep reporting overlaps.
- The figures appear in `LayoutStats` and in both hosts' snapshots. *Reading, not ruled:* when a spec travels with a score, an unmet pin maps to `unmet_holds`.

*Done when:*
- a pinned node reports honored;
- a planted unsatisfiable pair of pins (two bodies pinned to one point against contacts) reports unmet, the positive control;
- the anchor residual falls monotonically as `arrangement_pull` rises across a sweep;
- the reports reach the web snapshot and turnstone's observe snapshot;
- the eleven law receipts stay green.

## 3. Open questions

These are *Reading, not ruled*. Each returns to Mark at the named track's checkpoint, with evidence, if more than one answer is defensible.

1. **G1: what "relabel" covers.** This plan reads F7 as correcting docs and descriptive text. The picker label "Energy" stays plain under the 2026-09-02 ruling, "labels plain, ids technical". If the ruling meant the picker label, that is a change to put back.
2. **G1: the floor for F10.** This plan reads P2's floor, `energy >= 1`, as the measure.
3. **G1: the descent instrument's form** for terms that declare no energy (N). A loop-work check in joint configuration space, a Jacobian-symmetry check on small fixtures, or a persistent-motion check would each serve. The lane picks, or returns, with Kinds' positive control deciding which actually discriminates.
4. **G2: the registry and cartography's disclosures.** Whether the registry produces the disclosures too, or only shares channel ids with them.
5. **G3: the unit length for unary pulls.** `EdgeSpring`'s rest length, 170, is the obvious candidate. A per-term rest length is the other.
6. **G3: which overlays Density admits** after the conversion, and their bars.
7. **G5: which optimizer.** L-BFGS over the declared energy (OpenMM's minimizer, Penrose) or stress majorization (Graphviz's default for stress).
8. **The effectiveness record's home.** The brief, §8, argues for a table versioned beside the spec. This plan's Findings section is where it is seeded until a home is ruled.

## 4. Findings

- 2026-10-02 (planning): P5's rungs and Density's tiers are unmerged. `gpu-repulsion` is at `60a990a5`, and `density-cpu` is at `c402d5e7`, a merge of main over `5f529c64`. Density's overlay refusal is `PhysicsLaw::overlay_refusal` on that branch (`density-cpu:crates/canvas/pictograph/src/canvas/physics_catalog.rs`, line 199). G3 and G5 build on these once they land.
- 2026-10-02 (planning): `set_content_affinity` has no caller outside tests, and affinity is not in `PhysicsChoice` (physics catalog plan, the 2026-10-02 composition assessment; re-checked against main `d3874ff3`). G2 and G4 close both.
- 2026-10-02 (planning): the plan figure "0.77 alone … −0.54" for Density with `EdgeSpring` has no surviving log. The Density lane's first-round log was overwritten (the physics catalog plan's annotation of 2026-10-02). `probe-overlays.log` shows the same failure class for force overlays on Density, and is the one to cite.
- 2026-10-02 (planning): a correction to the brief's F9 evidence and §7.2, recorded here rather than in the brief's dated text. The brief called giving every member its group's full centroid force "not a gradient". More precisely, it is a gradient in a group-size metric: member forces are `n_g` times the true gradient, so it is class Em. It still descends the outer energy, but the total force on two groups of unequal size no longer cancels. It therefore fails reciprocity, which is what G3's positive control tests. The ruling (weight share) is unaffected. The rejected option is Em rather than N, and it mixes badly with identity-metric inner laws for the reason the brief gives for mixed metrics (§6.5).
- 2026-10-02 (planning): the brief's findings F-a to F-i stand as recorded there (brief, §5.7); G1 acts on F-a and F-f, G2 on F-g, and G5 on F-b.
- Effectiveness record, seeded (*Reading, not ruled* on its home; open question 8). The P2 receipts (`Code/testing/mere/physics_p2_receipt.md`), the P4 drag rows, and the density overlay probe (`Code/testing/mere/density/probe-overlays.log`; Density alone rank 0.60, with Hub pull −0.33 and 510 overlaps) are its first rows.

## Progress

- 2026-10-02: plan written from the brief and Mark's rulings F1 to F10, after merging main at `d3874ff3` into the brief's worktree. The physics catalog plan's P7 closed with a pointer here. No track started.
