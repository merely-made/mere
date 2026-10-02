# Batch 27 — dynamics grammar brief (new research brief)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/research/2026-10-02_dynamics_grammar_brief.md | current | yes | 169 | 169 | 0 | 0 |
| **Totals** |  |  | **169** | **169** | **0** | **0** |

**Totals: 1 doc, 169 claims checked (169 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `a059b838` (2026-10-02), with this pass's edits in the
working tree:
- the new brief;
- its `DOC_README.md` entry at the head of the `mere_docs/research/` list;
- a one-sentence pointer in the physics catalog plan's P7 "Held for a
  dynamics grammar" paragraph.

Branches `density-cpu` (`1bf47cf9`) and `gpu-repulsion` (`03cb4207`) were
read with `git show`; mer3ly at `7e455f9`; the P2 receipt and the density
probe logs under `Code/testing/mere/`. Web sources were fetched on
2026-10-02 and are listed in the brief's §12. `archive_docs/` is excluded.

This batch exists because the document is new. The record was taken after the
document was written and its citations spot-checked against the tree the same
day. Mark's 2026-10-02 rulings it carries are quoted from the physics catalog
plan, which is their primary record; they are counted as quotation checks, not
as claims about the tree.

## mere_docs/research/2026-10-02_dynamics_grammar_brief.md

- disposition: current
- status line: "Status (2026-10-02): research brief, complete; the forks in §11 wait for Mark. Docs only, no code changed." — accurate: yes
- claims checked: 169 — holds: 169, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none inside the brief. The brief records three disagreements elsewhere as
  findings rather than resolving them:
  - the physics catalog plan's 2026-09-02 finding that Kinds never rests, against
    `PhysicsLaw::never_rests` (`physics_catalog.rs:160-165`, since `37477457`);
  - `LinLogForce`'s doc, which describes LinLog's energy, against its kernel,
    which is ForceAtlas2's (1, −1) force model;
  - the mer3ly survey's `physics` wire field, against mer3ly `7e455f9`, which
    no longer carries one.

### Recommended action

- none for this record. The three findings above are forks F7 and F10, and
  finding F-h, in the brief; they are Mark's to rule.

### Notes

The claims checked, grouped by where each holds:

- **seiche core and slots**, 20: `Force` (`lib.rs:258`), `RepulsionRequest`
  (`:262-271`), `RepulsionSolver` and its threshold (`:364`, `:373`),
  `ForceContext` (`:378`), damping (`:218`), `set_forces` / `clear_forces`,
  the tick's force order (`:723-775`), `pin` (`:829-838`); `AnchorSpring`
  (`anchor_force.rs:7`, `:42`, `:83-105`); `AffinitySpring`
  (`affinity_force.rs:55`, `:91-122`); `CouplingForce` (`coupling_force.rs:38`,
  `:102-192`); `SceneField` (`scene_spec.rs:170`); Barnes–Hut (`barnes_hut.rs:42-48`,
  `:57`, `:89`); the tensor-forces module header.
- **`forces.rs`**, 7: `NodeExclusion`, `EdgeSpring` and `Boundary`, their
  defaults and their `apply` ranges.
- **The laws**, 34: `laws/mod.rs:65-83`; every struct, kernel and line range in
  the brief's §5.1 for Stress, LinLog (including its doc at `:9-11`), Gravity,
  ParticleLife, Boids, Kuramoto, MagneticSpring, Anneal (including the pinned
  skip at `:145-154`) and Hold.
- **The overlays**, 12: six structs and six kernel lines.
- **pictograph**, 28: `physics_catalog.rs` lines 56, 61, 62–65, 160–165, 169,
  242–249, 391–404, 416, 431–440, 443–449, 452–453, 456–460, 599, 719, 783–822,
  824–866, 994–1009, 1012–1071, 1075–1108, 1087, 1100–1106 and 1298–1323;
  `physics_board.rs:48`; `canvas.rs:75-97` and `:203`; `strategy.rs:208`,
  `:520-616` and `:576`.
- **cartography and sceno**, 17: `cartography_scene.rs` lines 146–161,
  197–199, 204, 226, 257 and 273; `adapters/mod.rs` lines 6–11, 243 and 345;
  `producers.rs:83` and `:139`; `parity.rs:466`; `adapters/score.rs:33`;
  `sceno/score.rs` lines 35–57, 85, 107–120 and 179.
- **Other in-tree**, 6: `SavedSceneV1` (`ports/graphshell/src/product.rs`, lines 216–243);
  numen's `NodeSelector` and `CouplingResponse` (`coupling.rs:69`) and
  `ScalarField` (`field_ast.rs:43`); seiche's manifest naming no rapier
  features (`Cargo.toml:19`); conatus's resident `integrate` (`kernels.rs:177`).
- **Branches and history**, 11: Density at `1bf47cf9` (`density.rs` lines
  7–15, 50, 294 and 469); two quotations from that branch's plan; the overlay
  probe's figures (`probe-overlays.log`, gen-200 rows); commits `438187cb`,
  `7784ddf5` and `03cb4207` on `gpu-repulsion`; `37477457` introducing
  `never_rests` without Kinds.
- **Siblings and receipts**, 3: mer3ly `7e455f9` `sceneState()`
  (`assets/graph-sandbox.js`, lines 1535–1558); the survey's wire table; the
  P2 receipt's results table (Kinds and Sync read at 1 s only).
- **Physics catalog plan quotations**, 17: the three P7 rulings; the currency,
  composition-tier, meaning and Bonds rulings; the Density + `EdgeSpring`
  figure; the P5 staleness and threshold rulings and the 2.9e-6 parity figure;
  the 2026-10-01 law-id, "Law in seiche" and "Ordinary laws with a CPU tier"
  rulings; the rapier-fallback quotation; the 2026-09-02 never-rests finding;
  the donor's Barnes–Hut quotation.
- **Projection grammar documents**, 14: catalog lines 43–57, 159–171, 357–400,
  427, 428, 430, 433 and 434; the adoption plan's lines 194–197 and 264–290 and
  its Findings table's first and last rows; the seam doc's Finding 1;
  DOC_README's "Design for known futures now" principle.

The brief marks three things it could not verify: the log behind the plan's
Density + `EdgeSpring` figure (the plan itself is cited), rapier's
cross-platform determinism, and HOOMD's rule on particles shared between
methods. None is counted above.

The judgment audit's coverage count before this pass was 308 of 328 active
documents; the 20 uncovered documents predate this batch and are not addressed
here.
