# Arrangement and dynamics brief

**Date:** 2026-10-03
**Status (2026-10-03):** assessment brief, complete; the forks in §8 wait for Mark. Docs only, no code changed. One measurement was taken with a temporary probe that was run once and not committed (§5).
**Ruling carried:** the [dynamics grammar plan](../implementation_strategy/2026-10-02_dynamics_grammar_plan.md), §1.1, F11 (2026-10-03). Asked how the physics side's tiers (dynamics grammar, dynamics recipe, domain binding) relate to the projection grammar's (projection grammar, scene recipe, domain binding), with the options nested, parallel peers, or one grammar, Mark answered: "Here is what I think. Physics acts on an arrangement. Physics is not assigning coordinates that nodes are pseudo pinned to return to once a force acts on them. Arrangements are positions, not motion. Physics layouts are about the rules by which things' movement is simmed/computed. Meaning can be encoded in the physics, too, like semantic grouping." Then: "I suspect this concept needs designing." The same day he ruled F12, "Dynamics recipe" (the middle tier's name), and F13, "README + plans" (where the vocabulary is written).
**Audit base:** mere `63c47a59`. Read only, not worked in: isometry `eaf155e`, mer3ly `abafbaf` (it moved from `4d20e05` during this pass; only its site-canvas plan changed), and the turnstone captures under `Code/testing/turnstone/images/scenarios/`. No network; the literature in §9 is cited from the record, not fetched.
**Measurement:** `Code/testing/mere/arr-dyn/probe.log`, with the probe's source beside it (`arr_dyn_probe.rs`), so the figures can be recomputed.
**Related:** [dynamics grammar plan](../implementation_strategy/2026-10-02_dynamics_grammar_plan.md) (F1 to F13), [dynamics grammar brief](2026-10-02_dynamics_grammar_brief.md) (the term classes and the anchor row), [projection proofs plan](../implementation_strategy/2026-07-21_projection_proofs_plan.md) (the 2026-07-23 entries, now annotated), [physics catalog plan](../implementation_strategy/2026-09-02_physics_catalog_plan.md) (P2 to P4, and the composition rulings), [projection grammar catalog](2026-08-15_projection_grammar_catalog.md) (placement policies, §6, the motion list), [projection grammar adoption plan](../implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md) (A6, A1, "Solver proposes, the score records"), [the cartography–gyre layout seam](../technical_architecture/2026-05-29_cartography_aether_layout_seam.md).

*Reading, not ruled* marks this brief's own inference. Every other claim is a quotation, a file and line, a measured figure, or a cited source.

**Path key.** `seiche/` is `crates/conatus/seiche/src/`; `pictograph/` is `crates/canvas/pictograph/src/canvas/`; `scenomise/`, `sceno/` and `scenotime/` are `crates/cambium/scenes/<name>/src/`; `graphshell/` is `ports/graphshell/src/`. "Catalog" is the projection grammar catalog, "proofs plan" the projection proofs plan, "physics plan" the physics catalog plan, "grammar plan" and "grammar brief" the dynamics grammar plan and brief, "seam doc" the cartography–gyre layout seam. Line numbers are at the audit base.

## 0. The result on one page

- **Who ruled the attractor.** Mark ruled that physics is "a global question that can apply to all layouts and arrangements" (proofs plan, line 368) and that "even swatches should be able to become physics-directed graphs" (line 391). Arrangement-as-attractor was an agent's design for carrying that out: a playing graph pulled back toward its arrangement's slots. The plan text that introduces it (lines 395–413) quotes no words of his, and neither do the two commits that landed it (`82ac70fe` and `12267085`, 2026-07-22 21:05). F11 reverses an agent design, not a ruling. Two later rulings of Mark's do lean on the anchor through the option text he chose: F1, "Terms and targets", and "Sequenced blends", whose option read "capturing positions as anchors". Both go back to him (A2, A3).
- **Inventory.** 36 places where an arrangement and the physics meet (§3): 18 consistent with F11, 10 in conflict, 8 ambiguous (§4). Two are in mer3ly: the sandbox's Anchored mobility, due to move onto the stack's board, and the portable fold fact, so reopening the attractor reaches mer3ly too.
  - The conflicts are one mechanism in five places: the canvas's `arrangement_pull`, the board's slot pull, the board's drag return, swatch relax's pull, and isometry's test-only use of relax.
  - Two more are its persistence and the catalog's §6 force row.
  - The last three are dynamics-grammar text built on it: the grammar brief's anchor row, capture-as-anchor, and G6's anchor residual.
- **What already agrees with F11:**
  - Picking an arrangement seeds the world from its positions and pauses.
  - Pausing freezes things where motion left them and never snaps back.
  - Returning is an explicit "Restore arrangement".
  - Switching is a scenotime transition, not a pull.
  - Pins are a person's holds, and every solver honours them.
  - `sceno::Hold::Anchored` already says "moving an anchored item is correct behaviour" (`sceno/score.rs:86-88`).
- **Measured** (§5). The fixture is seiche alone: 60 nodes and 74 edges under the canvas's default Springs trio, with both controls in the same run.
  - **Seed only loses the reading.** With the pull at 0, a Spiral's recency-to-radius rank falls from 0.999 to 0.30 at the 6 s settle budget and to 0.21 at 30 s; a random seed ends at −0.01. The "dissolved into an unrelated blob" receipt holds.
  - **The canvas default keeps it by force.** Pull 12 keeps 0.79, but it presses 11 pairs into overlap and holds edges at 235 against 178 free.
  - **A rule can carry it.** A term that reads recency as a radial depth, with no slot, keeps 0.61 with no overlaps and edges at their free length. Where structure follows recency it keeps 0.75, against the pull's 0.80.
- **The design space** (§6). An arrangement can act as:
  - a seed only;
  - constraints the motion respects (order, membership, separation, bounds);
  - facts that physics rules read, the route Mark named with "semantic grouping".
  
  A fourth case is positions that encode data, which may need a constraint on the encoded axis. Around these sit three handoffs (stop, switch, restore) and the question of recipe slots and the binding.
- **Forks** (§8), recommendation first:
  - A1: what "acts on an arrangement" means;
  - A2: F1's target class;
  - A3: capture-as-anchor;
  - A4: drag, and what "anchored" means;
  - A5: positions that encode data;
  - A6: the handoffs;
  - A7: recipe slots and the binding;
  - A8: the catalog's §6 force row.

## 1. The question

F11 was put as a choice among three relations between the two sides' tiers:
- nested: the dynamics grammar inside the catalog's "Force and constraint" arrangement family;
- parallel peers that meet only at the canvas;
- one grammar, with motion as one more operation.

Mark took none of them and said the concept needs designing (grammar plan, line 74).

The two sides' tiers, as they stand:

- **Projection side**, in Mark's words relayed with this commission (F13 writes them into mere's README):
  - **Projection grammar:** "the reusable vocabulary and operations: selection, derivation, visual encoding, arrangements, backgrounds, interactions, and provenance."
  - **Scene recipe:** "a particular composition of those choices, which can be saved, edited, and reused."
  - **Domain binding:** "which disclosed facts and permitted actions supply that recipe."
- **Physics side** (F12, grammar plan, line 76):
  - the dynamics grammar;
  - a dynamics recipe, a `DynamicsSpec` saved, edited and reused (G4);
  - a domain binding, `PhysicsChoice`'s successor. Whether this binding and the projection side's are one is left to this design.

What needs designing (*Reading, not ruled*): F11 fixes what an arrangement is (positions) and what physics is (rules of motion). It leaves open what, if anything, of an arrangement's reading should survive once the rules run, and by what route. The landed answer was a pull back toward the slots. F11 rules that out.

## 2. Who ruled arrangement-as-attractor

| Where | Date | Words | Whose |
|---|---|---|---|
| proofs plan, 368–371 | 2026-07-23 | "whether a given graph is affected by physics should be a global question that can apply to all layouts and arrangements... the idea of 'the mode with the physics' feels like a squandering." | Mark, quoted |
| proofs plan, 375–380 | 2026-07-23 | picking pauses by default; `apply_strategy_positions` "seeds the physics world ... so an arrangement is a real initial condition"; "playing does not pin the nodes" | agent, carrying out the line above |
| proofs plan, 389–392, repeated at 433 | 2026-07-23 | "even swatches should be able to become physics-directed graphs... a gesture, something like knocking twice on the graph's canvas." | Mark, quoted, under "Next rung (Mark's framing)" |
| proofs plan, 393–398 | 2026-07-23 | "The deeper form is **arrangement-as-attractor**: rather than a one-shot seed, an arrangement's targets become spring anchors ... where arrangements, fields, and physics become one design instead of three." | agent prose, three sentences after the quotation, with no quotation of its own |
| proofs plan, 399–413 | 2026-07-23 | "Arrangement-as-attractor landed — arrangements, fields, and physics are now one mechanism." | agent; the entry quotes no one |
| mere `82ac70fe`, `12267085` | 2026-07-22 21:05 | "seiche: anchor springs — an arrangement as a field, not an override"; "Arrangements pull as fields when the graph is playing" | commit messages, no attribution; fifteen minutes after `f79cde82`, "Physics is a global capability, not a layout mode" (20:50) |
| physics plan, 50–51, 102, 280–283, 296–300 | 2026-09-02/03 | "`AnchorSpring` (the arrangement-as-attractor pull)"; P3: "the projection's score is the seed and, through the anchor pull, an attractor" | plan text; the board's pull was the lane's design |
| physics plan, 428–433 | 2026-10-02 | Mark chose "Sequenced blends" (profiles with a schedule, and capturing positions as anchors) | Mark's words are the label; the anchor mechanism is the option's text |
| grammar plan, 33–36; grammar brief, 306 | 2026-10-02 | F1: "Terms and targets"; a target is "a slot set from a generator ..., held at a satisfaction class: encourage (the anchor) or ensure (pins and holds)" | Mark's label over the brief's definition |

**Verdict.** Mark ruled physics as a capability over every arrangement and every surface. The attractor was an agent's design for that, and it is presented in the plan as the design's deeper form, not as a ruling. F11 therefore does not overturn a ruling of Mark's on the attractor. It does touch two of his later rulings, F1 and "Sequenced blends", through the option text he picked. Those return to him as A2 and A3, with this finding.

**The receipt.** The landed entry reads: "the played Spiral now keeps its recognizable shape while relaxing, where the seed-only version dissolved into an unrelated blob" (proofs plan, 412–413). The only captures on disk are turnstone's `proof3_physics` set: `01_spiral_paused.png`, `02_spiral_played.png` and `03_free_graph_frozen.png`. They were written 2026-07-22 21:03, two minutes before the anchor commits. The played frame shows the fifteen nodes contracted toward the centre with labels overlapping, and it does not show which reading it is. No seed-only and anchored pair survives. The test that pins the design asserts anchor counts, not shape (`a_playing_arrangement_pulls_as_a_field_not_an_override`, `pictograph/tests/score_and_physics.rs:80`). §5 measures the claim.

## 3. Inventory

Every place an arrangement and the physics meet, with what pins it and whose words stand behind it. "None found" means the record was searched and holds no quotation of Mark's on that item.

### 3.1 The canvas

| # | Meeting point, and what it does | Where | Default | Pinned by | Mark's words behind it |
|---|---|---|---|---|---|
| I1 | **Seed.** Applying an arrangement buffers its slots and writes them into the physics world, with velocities reset, so a run starts from them. A transition's sampled frames do not seed; only its final placement does. Every apply re-seeds every listed body, so a slot cannot move without moving its body (physics plan, 1133–1135). | `pictograph/strategy.rs:146-168`, `:174-182` | on every apply | `arrangement_pause_freezes_motion_and_restore_is_explicit` ("restore must seed the simulation", `pictograph/tests/live_physics.rs:183`); `transition_preview_uses_the_strategy_buffer_until_the_final_snap` (`pictograph/tests/layout_and_drag.rs:49`); the `proof3_physics` captures | the capability ruling (proofs plan, 368) stands behind physics composing with every arrangement; the seed is the agent's implementation (377–379) |
| I2 | **A pick pauses.** Choosing an arrangement pauses through the visible flag, so the placement reads as placed; choosing none resumes. | `pictograph/strategy.rs:22-43` | pause on pick | `physics_is_global_and_composes_with_any_arrangement` (`pictograph/tests/score_and_physics.rs:119`) | as I1 |
| I3 | **Stopping stays.** Pause and play both re-seed from what is on screen, "never from stored arrangement slots"; anchors exist only while playing. | `pictograph/input.rs:596-626` | — | the I1 test ("pause must not restore", "resume must not snap"); `inline_play_pause_resume_changes_then_holds_then_changes_positions` (`live_physics.rs:155`); `elapsed_pause_resume_suspend_and_restore_never_catch_up` (`:103`) | none found |
| I4 | **Restore is explicit.** "Pause and return to the active arrangement's stored placement": a person's command, not a force. | `pictograph/strategy.rs:187-199`; `graphshell/canvas_controls.rs:23`, `:41-43`; the tree's button, `graphshell/web_tree/controls.rs:42` | — | the I1 test; scenarios `p4_tree_controls.scn` (lines 50, 82) and `p4_tree_elapsed.scn` (line 22) | none found |
| I5 | **The anchor pull.** While playing with a pull above zero, each body gets a spring toward its slot. The spring is applied after every other force, so stiffness reads as "how much does the arrangement win". The dial is copied into source-time snapshots. | `pictograph/strategy.rs:208-232`; field `pictograph/../canvas.rs:639-644`; default `pictograph/lifecycle.rs:212`; `seiche/anchor_force.rs:30` (stiffness), `:34` (slack), `:83-106`; tick order `seiche/lib.rs:806-811`; `pictograph/source_time.rs:173`, `:206` | 12.0 force per unit of offset; slack 0.5 | `a_playing_arrangement_pulls_as_a_field_not_an_override` (counts only); `seiche/anchor_force.rs:108-128` | none found; agent design (§2) |
| I6 | **Canvas drag.** A drag pins the body under the pointer and writes the drop point into the node's slot and the paused buffer. On release the body is unpinned and the anchors are rebuilt, so a playing node is then held near where it was dropped. | `pictograph/input.rs:81-96`, `:227-234`, `:376-389` | — | `dragging_a_node_updates_its_active_strategy_slot` (`layout_and_drag.rs:73`); `inline_playing_arrangement_holds_a_drag_then_releases_the_body` (`live_physics.rs:243`); the P4 drag rows (physics plan, 1213–1220) | "Seiche: skip non-dynamic bodies" (physics plan, 1221–1222) governs laws against a held body; the slot rewrite is agent design |
| I7 | **Pins and nudges.** `pin_focused` and `nudge_focused` hold a node as a kinematic body until `release_focused`. A drag does not undo an explicit pin. | `pictograph/input.rs:315-370`; `seiche/lib.rs:870-889` | — | `focused_keyboard_nudge_is_local_and_requires_explicit_release` (`layout_and_drag.rs:108`); `pointer_pull_preserves_an_existing_explicit_pin` (`:142`) | as I6 |

### 3.2 The physics board

| # | Meeting point, and what it does | Where | Default | Pinned by | Mark's words behind it |
|---|---|---|---|---|---|
| I8 | **Board seed.** One body per scene item. A new item spawns at its slot with a settle burst, and existing items keep their simulated positions. The score is never written. | `pictograph/physics_board.rs:171-216` | — | `items_hold_their_slots_and_a_new_one_joins_without_reseeding` (`physics_board.rs:371`); `physics_remote_board.scn` | none found (P3 is plan text, physics plan 280–283) |
| I9 | **Board slot pull.** Every slot is an anchor at the board's pull, so "the score is a seed and a soft attractor". | `physics_board.rs:39-44`, `:149-153`, `:312-322` | `DEFAULT_BOARD_PULL`, 12 / 24 = 0.5 | `charge_separates_overlapping_slots_and_orbit_never_rests` (`:474`, at 0.5); the I8 test at 12 | none found |
| I10 | **Board drag returns.** Release lets "the item ease back toward its arrangement slot". | `physics_board.rs:256-265` | — | `drag_pins_one_item_and_release_returns_toward_its_slot` (`:426`) | none found |
| I11 | **The practice board at pull 120.** The practice workspace places occurrences from data fields (`encoding.x` and `encoding.y` are `Field`s) and holds them at ten times the canvas default. Its module doc says physics "moves a transient view". | `graphshell/web_practice.rs:4-8`, `:201-206`, `:277-280` | 120 | not checked | none found |

### 3.3 The scene crates

| # | Meeting point, and what it does | Where | Default | Pinned by | Mark's words behind it |
|---|---|---|---|---|---|
| I12 | **Swatch relax's pull.** Repulsion, relation springs, and "a pull back toward the arrangement's own slots"; `untethered()` sets the pull to 0. Nothing in mere calls it. | `scenomise/relax.rs:7-23`, `:39-42`, `:56`, `:64-68`, `:272-278` | 0.25 per step (dt 0.1, velocity kept 0.82) | `the_arrangement_pull_holds_its_shape` (`relax.rs:496`) | Mark's swatch sentence (proofs plan, 433) stands behind swatch physics; the pull is agent design ("the `AnchorSpring` idea at swatch scale", 440–441) |
| I13 | **Relax keeps holds.** Honoured pins never integrate but still push their neighbours. Anchored holds relax "like anything else". | `scenomise/relax.rs:141-171` | — | `plain_relax_no_longer_drags_a_recorded_pin` (`:356`); `an_anchored_hold_still_relaxes` (`:384`); `a_held_item_does_not_move_however_crowded` (`:395`) | none found |
| I14 | **The score records positions and holds, never motion.** Score v4 holds an arrangement, items and holds. `Hold::Anchored` is "Best effort. The arrangement seeds from here and relaxation may carry it away; moving an anchored item is correct behaviour"; `Hold::Pinned` "Must be honored". Honoured and unmet holds are reported. | `sceno/score.rs:35`, `:85-92`, `:99-120` | — | `sceno/score.rs:709`, `:815-825`; the A6 and A1 receipts (adoption plan, 826–886) | none found; A6's "the mobility class ... is recorded rather than inferred" is plan text (adoption plan, 277–279) |
| I15 | **Switching is a transition.** A pick pauses (I2), and a scenotime schedule carries the view from the current geometry to the new slots. Only the final slots seed the world. | `graphshell/canvas_physics.rs:92-145`, `:151-166`, `:181-258` | `TransitionSpec::default()` | the I1 transition test | none found |
| I16 | **`ReturnMotion`.** A host-clocked spring that returns a label or callout's local offset home, with modes Home, Free and Pinned; it "does not move a scene item". It is used by isometry's overmap overlay, interaction and motion state. | `scenotime/return_motion.rs:7-13`, `:56-65` | strength 18, damping 0.78 | `scenotime/return_motion.rs:281`, `:291` | none found (`c4ef60eb`, 2026-09-09) |

### 3.4 Graphshell

| # | Meeting point, and what it does | Where | Default | Pinned by | Mark's words behind it |
|---|---|---|---|---|---|
| I17 | **`SavedSceneV1.arrangement_pull`.** The canvas's dial is saved and reopened with the scene. Like `physics_paused` and `physics_damping`, and unlike the five physics-catalog fields, it has no serde default, so a scene saved without it does not open. | `graphshell/product.rs:238`; read at `graphshell/web_product.rs:230`, written at `:502`; copied at `graphshell/transfer.rs:987` | the canvas's value; fixtures use 0.4 (`product.rs:827`, `:953`; `graphshell/app.rs:717`; `graphshell/personal_sync.rs:1777`) | `saved_scene_physics_fields_default_and_round_trip` (`product.rs:820`) | none found |
| I18 | **Free (physics alone).** The picker's no-arrangement entry. It was added because "The boot Spiral's anchor springs pulled under every law, holding Stress at a stretch of 1.45 and squeezing Flow into seven overlaps". | `graphshell/canvas_physics.rs:22-33`, `:101-111`; physics plan, 266, 775–779 | — | `Code/testing/mere/physics_p2_receipt.md`, lines 17 and 50–53 | none found |
| I19 | **Controls.** The tree's canvas controls offer "Restore arrangement" and "Play/Pause physics". No control anywhere in Graphshell sets the pull. | `graphshell/web_tree/controls.rs:25-50`; `graphshell/canvas_controls.rs:12-46` | — | the I4 scenarios | none found |

### 3.5 The governing documents

| # | Meeting point, and what it does | Where | Mark's words behind it |
|---|---|---|---|
| I20 | Catalog **Free** policy: "the solver supplies a stable initial arrangement; direct manipulation may establish a new position"; the intent "Drag free item". | catalog, 63 and 258 | none found (lines arrived in the sweep commit `6c37466b`, 2026-08-16) |
| I21 | Catalog **Anchored** policy: "displacement is temporary and the item returns toward home"; the intent "Displace anchored item", "then return to solved home"; the Penrose transfer, "an anchored home is encourage-class". | catalog, 64, 259 and 434 | none found |
| I22 | Catalog **Pinned** policy and the intent "Pin or unpin". | catalog, 65 and 260 | none found |
| I23 | Catalog §6, the **"Force and constraint"** arrangement family: "Spring, collision, gravity, clusters", "Landed in Seiche". | catalog, 217 | none found |
| I24 | Catalog's **motion meanings**: "solver motion toward an arrangement; direct manipulation; transition between projections or epochs; data-encoded motion; ambient product behavior", which "should not collapse into one physics switch". | catalog, 264–277 | none found |
| I25 | The **seam**: cartography strategies "*compute* a layout", gyre "*simulates* one", and they meet through a seed and read-back pair. The physics catalog's module doc says the arrangement catalog "is *where* nodes go, this is *how* they move". | seam doc, 41–49 and 70–81; `pictograph/physics_catalog.rs:23-30`; carried under F1 at grammar plan, line 18 | none found |
| I26 | Grammar brief's **anchor row**: "Anchor (arrangement pull)", class "E. An encourage-class *target* term"; and §6.3: the anchor spring "turns any arrangement into an encourage-class energy term". | grammar brief, 173 and 291 | none (the brief's analysis) |
| I27 | **F1's target class**: "A slot set from a generator ..., held at a satisfaction class: encourage (the anchor) or ensure (pins and holds)". | grammar plan, 33–36; grammar brief, 83, 306 and 315 | Mark: "Terms and targets" |
| I28 | Grammar brief's **"Pins (drag)"** row: an ensure-class constraint, with "No satisfaction is reported". | grammar brief, 178 | "Seiche: skip non-dynamic bodies" |
| I29 | **Centre and Tide**: unary pulls toward a fixed point and toward a point moving on a sine. Both are physics overlays; neither target comes from an arrangement. | grammar brief, 163–164; `seiche/overlays/gravity_locus.rs:36`, `:46` | none found (they map the donor's presets, physics plan, 109–117) |
| I30 | **G3's capture-as-anchor**: "'Capture positions as anchors' freezes one stage's layout as the next stage's target, through the anchor slot". It is done when "a sequenced blend reproduces the captured anchor layout within a stated tolerance". | grammar plan, 160 and 170; physics plan, 432–433 and 663–665; grammar brief, 327 | Mark: "Sequenced blends"; the anchor mechanism is the option's text |
| I31 | **G4's target reference**: the spec carries "a target reference (arrangement plus satisfaction class)"; the brief's sketch reads `Encourage { pull } or Ensure`. | grammar plan, 177; grammar brief, 415–416 | follows F1 |
| I32 | **G6, pins and contacts**: pins report honoured or unmet; contacts keep reporting overlaps. | grammar plan, 213 and 215 | follows F1 |
| I33 | **G6, the anchor residual**: anchors report residual RMS, and the track is done when "the anchor residual falls monotonically as `arrangement_pull` rises across a sweep". | grammar plan, 214 and 221 | follows F1 |

### 3.6 Other repositories (read only)

| # | Meeting point, and what it does | Where | Mark's words behind it |
|---|---|---|---|
| I34 | isometry's `overmap_positions_relaxed` runs `scenomise::relax` before the swatch fit. The product path, `overmap_positions`, passes `None`, so only its tests relax. | `isometry/crates/isometry-views/src/overmap/scene.rs`, lines 62–80; tests at 143 and 158 (isometry `eaf155e`) | the swatch sentence, as I12 |
| I35 | mer3ly's sandbox **`GraphPhysics` mobility**, a visitor's choice of "Anchored" or "Free":<br>• "Anchored", the default, re-seeds every body at the arrangement's targets and installs `AnchorSpring` at 13.0 toward them; "Free" installs no anchor. Manual pins keep their current point either way.<br>• A pin under Anchored is exported as `Hold::Anchored`, otherwise as `Hold::Pinned`, and the class is shared as `mer3ly.motion`.<br>• The site-canvas plan lists "`GraphPhysics`: pins, the anchored/free split, backdrops" as a site-local behaviour to retire by consuming "`seiche`, and `mere::canvas::PhysicsBoard`", verdict "consume". | `mer3ly/assets/graph-sandbox.js` (tracked; the `.tmp/` copies are ignored): default at 252, restored at 348–351, 417–418 and 476–477, offered at 681–684, set at 743–744, handed to `GraphPhysics.setArrangement` at 820, exported at 1521 and 1555. `mer3ly/crates/repo-graph/src/lib.rs`: `set_arrangement` at 380, which applies at 1615–1660, with the anchor at 1637–1646 (13.0 at 1644). `mer3ly/docs/2026-09-30_graphshell_site_canvas_plan.md`, line 774. All at mer3ly `abafbaf`. | Mark's Ruling 1 stands behind the move onto the stack: "the site counts as a consumer, but it should be consuming or creating stack capabilities, not special exceptions" (plan, 37–38). None found behind the anchor itself, or behind Anchored as the default. |
| I36 | mer3ly's **portable fold fact naming its stand-in**, due on sceno's 0.0.4 line:<br>• The fact "carries its members plus a stand-in": either one of the members ("the site's root with a '+N' badge") or a synthetic summary ("pictograph's body"). The host picks per fold, and every reader renders whichever the fact names.<br>• Today mer3ly's fold hides the members and tags the root with a `fold` channel, with no re-solve, so the root stands in at its own position.<br>• Mere's canvas fold puts the summary "at the centroid of its members' existing layout positions" and "never moves or re-seeds those source members"; "The source graph and its physics have never changed". The summary is not yet a physics body.<br>• Stage S5 carries the work. | `mer3ly/docs/2026-09-30_graphshell_site_canvas_plan.md`: Ruling 11 at 139–152, Ruling 20 at 233–241, S5 at 1193–1205. `mer3ly/crates/repo-graph/src/lib.rs`, `visibility_diff` at 2310–2340. `pictograph/fold_projection.rs:7-11`, `:65-80` and `:192-193`. | Mark, Ruling 11: "Canvas adopts + portable (Recommended)". Ruling 20: "Fact names its stand-in (Recommended)". Both rule fold semantics, not physics. |

## 4. Against F11

| # | Class | Why |
|---|---|---|
| I1 | consistent | Physics starts from the arrangement's positions: it "acts on an arrangement". |
| I2 | consistent | The arrangement shows as positions; motion is opt-in. |
| I3 | consistent | Stopping leaves bodies where motion left them, and nothing pulls back. |
| I4 | consistent | The return is a person's command that re-asserts positions, not a force. |
| I5 | conflict | Exactly "coordinates that nodes are pseudo pinned to return to once a force acts on them". |
| I6 | ambiguous | Writing the drop into the slot treats the arrangement as positions a person edits, which is consistent; the hold at the drop point that follows is I5's pull. |
| I7 | consistent | A pin is a person's hold, not physics assigning coordinates; physics acts around it. |
| I8 | consistent | New items start at their slots, and existing ones keep what physics gave them. |
| I9 | conflict | I5's mechanism, gentler. |
| I10 | conflict | The plainest case: released, the item is pulled back to its coordinates. |
| I11 | ambiguous | I5's mechanism, but the positions encode data fields, so the pull is fidelity to an encoding rather than to an arrangement (A5). |
| I12 | conflict | I5 at swatch scale. |
| I13 | consistent | Holds outrank physics. The anchored items it relaxes are carried home only by I12's pull. |
| I14 | consistent | The portable record holds positions and holds, never motion, and `Hold::Anchored` already lets motion carry an item away. |
| I15 | consistent | Switching is a transition between positions, kept apart from physics. |
| I16 | ambiguous | The same return-home shape, but over a label's local offset on a host clock, not node placement under physics. |
| I17 | conflict | It persists I5's stiffness as scene state. |
| I18 | consistent | Physics acting with no arrangement; its origin is evidence that I5 masked the laws. |
| I19 | consistent | The exposed controls are pause, play and restore, and the pull is not one of them. |
| I20 | consistent | An initial arrangement, a new position by manipulation, and no return. |
| I21 | ambiguous | "Returns toward home" is the rejected shape if physics does the returning. As a per-item interaction policy it may not be physics at all. And `sceno`'s `Hold::Anchored` (I14) defines anchored the opposite way. |
| I22 | consistent | An explicit constraint set by a person. |
| I23 | conflict | It lists live physics as an arrangement family, against "Arrangements are positions, not motion". |
| I24 | ambiguous | "Solver motion toward an arrangement" is either the attractor or a solver converging on an arrangement's positions, shown as motion. The rest of the list, and its refusal of one physics switch, agrees with F11. |
| I25 | consistent | Compute positions, then simulate from them: the order F11 states. |
| I26 | conflict | It classes the attractor as the bridge between arrangements and physics. |
| I27 | ambiguous | Mark's own ruling, whose target class was defined by the anchor (encourage) and pins (ensure). The ensure half agrees with F11; the encourage half does not (A2). |
| I28 | consistent | A pin is an ensure-class hold. |
| I29 | consistent | A locus is a rule of motion with its own target, not an arrangement's coordinates. Its kernel is I5's; the source of its target is not (§6.1). |
| I30 | conflict | It freezes a layout as coordinates the next stage is pulled back to, under a label Mark chose (A3). |
| I31 | ambiguous | It follows I27: `Ensure` agrees, and `Encourage { pull }` is I5. |
| I32 | consistent | It reports on holds and contacts. |
| I33 | conflict | It presupposes I5's dial. |
| I34 | conflict | It inherits I12, in tests only. |
| I35 | ambiguous | Anchored is I5's mechanism, and the default; Free agrees; the visitor chooses. Under Ruling 1 it can only consume what the stack offers, so A1 and A4 decide what it becomes. |
| I36 | consistent | The fact records membership and a stand-in, positions and identity, never motion. Neither stand-in is held to a coordinate: the root keeps its own position, and the summary's centroid follows its members wherever motion has them. It meets the physics only where a host simulates it: a board would anchor a stand-in at its slot (inheriting I9), and the canvas's hidden members keep their bodies (*Reading, not ruled*). |

**Counts:** 18 consistent (I1–I4, I7, I8, I13–I15, I18–I20, I22, I25, I28, I29, I32, I36), 10 conflict (I5, I9, I10, I12, I17, I23, I26, I30, I33, I34), 8 ambiguous (I6, I11, I16, I21, I24, I27, I31, I35).

## 5. Measurement: how much of a Spiral survives

The claim at stake is the landed design's receipt: seed-only "dissolved into an unrelated blob", while the pull kept the shape. No artifact backs it (§2). The probe measures it, and tries one rule that F11 allows.

**Probe.** A temporary integration test in seiche, built offline and `--locked` into `C:/t/cargo-targets/mere/arr-dyn` and run once; its source is kept beside the log, not in the tree.
- **Graph.** 60 nodes. The edges are a random recursive tree (59 edges) plus 15 cross edges, from one fixed seed. Two assignments of recency ordinals:
  - *independent*: the ordinals are shuffled, so recency says nothing about structure. This is the case in which a Spiral dissolves.
  - *trail*: recency is growth order, as a browse trail grows.
- **Physics.** The canvas's default trio, `NodeExclusion`, `EdgeSpring` and `Boundary` at their defaults, as `pictograph/seiche_bridge.rs:58-60` builds them. Seiche's damping is 2.5 and dt is 1/60, on the CPU, with no couplings or affinity.
- **Slots.** The scenomise Spiral formula: radius = spacing · √ordinal, at the golden angle (`scenomise/solve.rs:296-311`). The spacing is 57.6, the canvas's 36-pixel face times 1.6, which replaces Spiral's default of 22.
- **Measures.**
  - ρ: the Spearman rank correlation between ordinal and distance from the centroid. This is the Spiral's reading, recency as distance from the centre.
  - The mean edge length; `EdgeSpring`'s rest length is 170.
  - Overlapping pairs: centres closer than 36.
  - Kinetic energy.
  
  All are read at 360 ticks (`SETTLE_TICKS`, the 6 s budget) and at 1 800 (30 s).
- **Couplings.**
  - The anchor pull at 0, at 0.5 (the board's default), at 2, at 12 (the canvas's default) and at 120 (the practice board's).
  - A probe-local **radius band**: a unary spring on distance from the centre toward spacing · √ordinal, with the angle free. It is a rule that reads recency as a radial depth, class E, written for the probe from §6's description.
- **Controls, same run.**
  - Positive: the Spiral seed itself reads ρ 0.999, asserted above 0.99.
  - Negative: a uniform random seed reads 0.120 (independent) and −0.154 (trail) before physics, and −0.007 after 30 s with no pull (independent).

  The instrument can say both yes and no.

| Coupling | Independent ρ, 6 s | ρ, 30 s | edge, 30 s | overlaps, 30 s | Trail ρ, 30 s | edge, 30 s | overlaps, 30 s |
|---|---:|---:|---:|---:|---:|---:|---:|
| Spiral seed, before physics (positive control) | 0.999 | — | 424.8 | 0 | 0.999 | 368.5 | 0 |
| Random seed, pull 0 (negative control) | 0.087 | −0.007 | 177.6 | 0 | 0.456 | 177.6 | 0 |
| Pull 0 (seed only) | 0.304 | 0.207 | 177.5 | 0 | 0.375 | 175.7 | 0 |
| Pull 0.5 (board) | 0.531 | 0.465 | 181.2 | 0 | 0.494 | 180.5 | 1 |
| Pull 2 | 0.516 | 0.523 | 189.5 | 2 | 0.634 | 189.8 | 2 |
| Pull 12 (canvas) | 0.790 | 0.790 | 234.6 | 11 | 0.796 | 228.6 | 10 |
| Pull 120 (practice board) | 0.965 | 0.965 | 358.3 | 1 | 0.971 | 320.4 | 1 |
| Radius band, k 2 | 0.580 | 0.610 | 180.0 | 0 | 0.754 | 176.9 | 0 |
| Radius band, k 12 | 0.695 | 0.583 | 187.0 | 1 | 0.745 | 179.6 | 2 |

What it shows (*Reading, not ruled*, scoped to this fixture):

- **The blob is real.** With no pull, the Spiral's order falls to 0.30 within the settle budget and to 0.21 by 30 s, a third of the way from the seed to the random control. On the trail graph the seed's order is gone by 30 s: 0.38, against 0.46 reached from a random seed.
- **The canvas default keeps the order by holding the physics back.** At pull 12, edges stay at 235 against 178 free, so 23% of the seed's excess length survives. The anchors press 11 pairs into overlap, and pull 0 leaves none. That is the P2 receipt's finding (Stress at 1.45 against about 3.5, Flow in seven overlaps) at a second site. At 120 the physics barely shows: 73% of the excess survives.
- **A rule can carry part of the reading with the physics fully expressed.** The radius band holds 0.61 on the independent graph, with edges within three units of free length and no overlaps. On the trail graph it holds 0.75, within 0.05 of pull 12's 0.80. It is weaker than the pull where structure says nothing about recency, and near it where structure agrees.
- **Where structure follows recency, the law already carries part of the reading.** Physics alone, from a random seed, reaches 0.46 on the trail graph.
- **The band is not tuned.** At the settle budget it is still moving: kinetic energy 15 960 (k 2) and 20 345 (k 12), against 22 for pull 12. At k 12 it rings. A rule of this kind needs F5's common-scale calibration and a settle check before any receipt.

Limits: one fixture, one seed, 60 nodes, seiche alone rather than the canvas or a host. The band is a sketch to size a route, not a proposal for the code.

## 6. The design space

### 6.1 What F11 fixes and what it leaves open

*Reading, not ruled.* Mark's words fix four things:
1. An arrangement is the positions it computes; it is not a motion.
2. Physics is rules of motion, and it starts from the positions an arrangement gave.
3. Physics does not hold bodies to coordinates they are pulled back to after a force moves them.
4. Meaning may live in the rules, and "semantic grouping" names one: P7c's first instance, Charge between meaning clusters with springs within (physics plan, 658), is G3's Groups.

Open: what of an arrangement's reading should survive once the rules run, and by what route.

**The kernel is not the question.** A unary pull toward a point is one kernel, used five ways in the tree:
- the arrangement's anchor (I5);
- Centre and Tide (I29);
- Depth, `s(depth·h − x·â)·â`;
- Group pull, toward a centroid;
- Grid snap, toward the nearest lattice point (grammar brief, §5.2).

F11 does not forbid the kernel. It rules on where the target comes from. A rule that computes its target from facts (a depth, a centroid, a field, a moving locus) is physics. A coordinate an arrangement assigned is not. That is why §4 classes Centre and Tide as consistent and the anchor as conflict. It also says what F11 does to the grammar's target class: it removes the class's main member, the anchor, and leaves its kernel to the rules.

### 6.2 Four routes

| Route | Operationally | Keeps | Costs |
|---|---|---|---|
| **A. Seed only** | The arrangement's positions are written into the world once (I1 does this already), and no term refers to them afterwards. The pull is 0 everywhere and relax is untethered. | F11 to the letter. The code exists (I1, I3, `untethered()`). The spec needs no target class. Laws show their signatures under any arrangement, so the P2 masking goes. | The reading decays under play: 0.999, then 0.30 at 6 s and 0.21 at 30 s (§5). An arrangement reads while paused and through the first seconds of a run. |
| **B. Constraints the motion respects** | Each arrangement family declares which relations among positions carry its reading. Physics moves bodies freely inside the set those relations allow. The relations: **order** (ordinal to radius for Spiral, time to x for Timeline, rows and columns for Grid), **membership** (a column's x interval for Columns, a ring's annulus for Radial), **separation** (minimum gaps) and **bounds**. | The reading as relations, with no coordinate fixed. Physics is expressed inside the allowed set. Satisfaction becomes a per-constraint report, which extends G6 and sceno's honoured and unmet holds. | A constraint compiler per family. Generators have weak constraint forms: Spiral reduces to radius order, Penrose and Fractal to nothing beyond their positions. A new term family and a solver step in seiche. Constraints can be unsatisfiable against contacts, and must then report, as the ensure discipline requires. |
| **C. Facts as channels for rules** (meaning in the physics) | The arrangement's inputs, the per-item values it discloses, become channels in G2's registry, and physics terms read them: recency as a radial depth (§5's band), site or cluster as groups (G3's Groups, Group pull by any channel, F-g), time as an axis depth, an embedding as content-affinity pairs. The arrangement then contributes only the seed. | Mark's "Meaning can be encoded in the physics, too". Every term stays a rule, class E for depth and group terms, so the dynamics grammar needs no target class for these families. Arrangements and physics share facts, not coordinates. G2 already plans the first twin, "Columns (by cluster)" (grammar plan, 148). | A twin exists only where a channel carries the reading. The grammar brief's §5.4 lists none for Spiral, Penrose or Fractal, and Grid's overlay picks the nearest cell, not the ordinal one. A twin carries the reading, not the look: a centroid, not a column. Measured: the band keeps 0.61 (0.75 on the trail graph) against the pull's 0.79 (0.80). |
| **D. Positions that encode data** | Where position is an encoding, not an arrangement's choice (the practice board's `Field` channels, I11; the catalog's Scatter scene, where "quantitative values determine position"), the encoded axis is held and physics moves only the free axes. With both axes encoded, physics only separates overlaps. | Fidelity to data, without an arrangement assigning anything. This is the shape of collision-dodged dot plots (Wilkinson 1999) and of d3's beeswarm idiom: an axis pull plus collision on the free axis. | One more source of constraints, declared from the encoding rather than the arrangement. Holding an encoded axis stiffly is mathematically the same quadratic as a pull, so the difference is where it is declared and what it means (*Reading, not ruled*). |

**B's literature.** Constraint layout separates what must hold from where things are:
- **Separation constraints.** IPSep-CoLa (Dwyer, Koren and Marriott 2006) solves inequalities of the form x_i + gap ≤ x_j inside stress majorization, by gradient projection. Such a constraint fixes order and gaps, never a coordinate, which is "constraints the motion respects rather than positions to return to" in its oldest form.
- **Hierarchy.** Dig-CoLa (Dwyer and Koren 2005) expresses hierarchy as y-separation constraints.
- **Projection in a force loop.** Dwyer (2009) applies the same projection inside a force-directed loop, which is the realization closest to seiche's.
- **Constraints inside stress majorization.** Wang et al. (2018) treat interactive constraints as part of stress majorization.
- **Earlier constraint layout.** He and Marriott (1998), and Kamps, Kleinz and Read (1995).
- **Set scope.** SetCoLa (Hoffswell, Borning and Heer 2018) scopes such constraints to predicate-defined sets, so one authored constraint reapplies to a second graph.
- **What a layout should keep.** Misue, Eades, Lai and Sugiyama (1995) name the properties a mental map keeps: orthogonal ordering, proximity and topology. That is a vocabulary for what B should preserve.
- **Compliance.** XPBD's compliance (Macklin, Müller and Chentanez 2016) runs from a hard constraint at zero to a soft one, and Penrose's `ensure`/`encourage` names the same split.

In seiche, B has three candidate realizations:
- a one-sided penalty that acts only when a relation is violated, of which `AffinitySpring`'s one-sided spring (grammar brief, §5.3) is the shape;
- a projection step after integration, like the state-writers' skip of non-dynamic bodies, which shows seiche already has post-step writers;
- gradient projection inside G5's optimizer, for compositions made entirely of energy terms.

**C and B combine.** C carries a reading as a force; B carries it as a bound. A family can use both: Columns as groups by site (C) plus a column interval (B). The band in §5 is C, a rule fed by a channel. A strict radius-order constraint would be its B form.

### 6.3 When physics stops

Today stopping stays and return is explicit:
- Pause, and the end of a settle budget, leave bodies where motion left them.
- `set_physics_paused` re-seeds "from what the user sees, never from stored arrangement slots" (I3).
- The I1 test asserts "pause must not restore" and "resume must not snap".
- The return is "Restore arrangement", a person's command that pauses and re-asserts the positions (I4).

*Reading, not ruled:* this is what F11's words imply. The alternative is that stopping returns by a transition. That would keep "arrangements are positions", since the return is a scenotime transition and not a force, but it makes every pause undo the motion (A6).

### 6.4 Switching arrangements while playing

A pick pauses (I2). A scenotime schedule carries the view from the current geometry to the new slots, and only the final slots seed the world (I15).

Under F11, a switch is a transition between positions. It is never a physics pull toward the new slots, because that is the attractor. The open part is whether the play state survives a pick: run the transition, then resume physics from the landed positions (A6).

### 6.5 The Spiral without pinning

Ways to meet "the seed-only version dissolved into an unrelated blob" with no pull:
1. **Accept it.** The Spiral reads while paused (the default on pick), and play hands it to the law. This is route A.
2. **A rule over recency.** A radial depth fed by the recency channel, which §5 measured at 0.61 (0.75 on the trail graph) with no overlaps. This is route C.
3. **A radius-order constraint.** Older inside newer, with physics free otherwise. This is route B, and it is not measured here.
4. **The trajectory as the reading.** The first seconds of relaxation are the picture: the dynamics grammar's H-class view, read at a stated time.
5. **A settled result recorded as positions.** Run the law to rest and record that as an arrangement. This is the converged pass the seam doc's Finding 2 kept for headless use, and A8's option 2.

### 6.6 Recipe slots and the binding

Evidence:
- **The scene recipe has no motion slot.** `scenograph::ProjectionDefinition` has source, reading, encoding, arrangement, interaction, appearance and provenance (`crates/cambium/scenes/scenograph/src/lib.rs`, lines 240–251).
- **Physics lives in host state.** On or off is `PracticeRuntimeConfig { layout_id, physics_enabled }` (`graphshell/practice_workspace.rs:224-227`), and the canvas's choice is `SavedSceneV1`'s flattened physics fields (I17).
- **G4's carrier.** G4 carries the `DynamicsSpec` in `SavedSceneV1` (grammar plan, 177–181).
- **The binding's text.** Mark's definition of a domain binding is "which disclosed facts and permitted actions supply that recipe".
- **The physics side reads the same facts.** The physics channels pair with cartography's disclosures (grammar brief, §5.5): kind with the categorical axis, mass with weight, depth with the numeric axis.
- **The plan already asks.** The grammar plan's open question 4 asks whether G2's registry produces cartography's disclosures too.

*Reading, not ruled:* under routes B and C the arrangement and the dynamics recipe read one set of facts, which argues for one binding. Physics also reads facts no arrangement reads (PageRank mass, meaning pairs), which argues for a binding that is a superset, or two. A7 puts the choice.

## 7. Consequences

What each route does to the places F11 touches. *Reading, not ruled* throughout.

The catalog consequences (I20–I24, the catalog lines in A4, and A8) are not carried from here. The Projection grammar session holds its edit to the catalog's §6 until this design rules, then carries the result into the catalog and the adoption plan itself (coordination, 2026-10-03).

| Route | Target class and F1 | G3 capture-as-anchor | G6 reports | `SavedSceneV1` | Swatch relax | Catalog §6 row |
|---|---|---|---|---|---|---|
| **A. Seed only** | A target is a seed reference; `Ensure` holds stay. | Becomes capture-as-seed; the done-condition at grammar plan, 170, is rewritten. | The anchor rows (214, 221) drop; pins and contacts stay. | `arrangement_pull` is retired. It needs a `#[serde(default)]`, which it lacks today (`product.rs:238`), so saved scenes still open; the value is ignored. | `untethered()` becomes the only behaviour and the 0.25 default goes. isometry's product path is unaffected (I34). | The row leaves the arrangement table (A8). |
| **B. Constraints** | Targets become derived constraints, each ensure or encourage, with a compliance. | Capture freezes relations (order, proximity) as the next stage's constraints. | A residual per constraint, and honoured or unmet per ensure relation, as sceno's holds already report. | The pull is replaced by the dynamics recipe's constraint settings (G4). | Relax gains a dependency-free constraint projection. | §6 keeps arrangement-derived constraints as constraints on positions; live physics leaves. |
| **C. Facts as channels** | Seed only for targets; the meaning moves to energy terms over channels, class E. | Capture-as-seed; the next stage's rules carry what should be remembered. | The anchor rows drop; each channel term reports its signature observable, for example ρ for a recency depth. | The pull is replaced by the recipe's terms (G4). | Relax would need channel terms, or swatches stay seed only. | The row leaves; G2's registry is where arrangements and physics meet. |
| **D. Encoding-bound** | A narrow ensure target survives on encoded axes. | Not affected. | A residual on the encoded axis. | The practice board's 120 is not saved today; it would be a recipe setting. | Not affected. | An encoding matter, outside §6. |
| **Opt-in pull** (A1, option 4) | `Encourage { pull }` stays, default 0. | Capture-as-anchor stays available. | The anchor residual stays for scenes that turn the pull on. | The field stays; new scenes save 0. | Relax's default moves to 0. | As A8 rules. |

## 8. Forks for Mark

Each fork gives its evidence with numbers, then two to four options saying what each commits to, recommendation first. The fork labels are A1 to A8, so that they do not collide with the grammar plan's F1 to F13.

**A1. What "physics acts on an arrangement" means.**
Evidence: with the pull at 0, the Spiral's order falls from 0.999 to 0.30 within the settle budget and to 0.21 at 30 s. The canvas's pull of 12 keeps 0.79, but it presses 11 pairs into overlap and holds edges at 235 against 178 free. A rule reading recency as a radial depth keeps 0.61, or 0.75 where structure follows recency, with no overlaps and edges at free length (§5). Every family has a seed; only some have a channel twin (grammar brief, §5.4).
1. **Seed, then rules that read the arrangement's facts** (recommended). The arrangement is the initial condition. What should survive of its reading reaches physics as rules over the channels it discloses, through G2's registry: recency as a radial depth, site or cluster as groups, time as an axis depth. No term pulls toward a slot. This commits to retiring the anchor pull (I5, I9, I10, I12), and to G2 growing a twin rule wherever one carries a family's reading. Families with no twin (Penrose, Fractal) are seed only. Constraints (option 2) can follow as a later track.
2. **Seed, then constraints derived from the arrangement.** Each family compiles the relations that carry its reading (order, membership, separation, bounds) into constraints the motion respects, as IPSep-CoLa does. This commits to a constraint term family, a projection or one-sided-penalty step in seiche, and a compiler per family. The look survives better than under option 1: columns stay columns.
3. **Seed only.** Nothing derived from arrangements acts during a run. An arrangement reads while paused and through the first seconds of play. This commits to retiring the pull and adds nothing.
4. **Keep the pull as an opt-in tunable at default 0.** The seed reading by default, and the attractor on request, labelled as holding the arrangement. This is the smallest code change (the default moves from 12 to 0), and the conflict remains wherever someone turns it on.

**A2. F1's target class.**
Evidence: F1, "Terms and targets", defined a target as "a slot set from a generator ..., held at a satisfaction class: encourage (the anchor) or ensure (pins and holds)" (grammar brief, 306). F11 removes the encourage realization. The ensure half (pins, honoured holds) is consistent (I28, I32). G4's sketch carries `Encourage { pull } or Ensure` (grammar brief, 415–416).
1. **Re-read targets as the arrangement reference that seeds a run, plus the holds that outrank it** (recommended). F1's three kinds of term stand. The target kind keeps `Ensure` (pins, honoured holds, and A1's constraints if option 2 is chosen) and loses `Encourage { pull }`. This commits to a dated amendment under F1, and G4's target reference becomes "seed arrangement plus holds".
2. **Drop the target kind.** The model has energy and dynamics terms. Arrangements are referenced as seeds outside the term list, and pins become a realization constraint. This commits to rewriting F1 from "Terms and targets" to terms alone.
3. **Keep encourage targets as an opt-in term.** This pairs with A1's option 4.

**A3. Capture-as-anchor in G3's schedules.**
Evidence: Mark chose "Sequenced blends", whose option text read "profiles with a schedule, and capturing positions as anchors" (physics plan, 432–433). G3 is done when "a sequenced blend reproduces the captured anchor layout within a stated tolerance" (grammar plan, 170). The grammar brief's example is "Stress, then capture, then Springs: the metric map, relaxed locally, remembering it" (§7.3).
1. **Capture as seed** (recommended). The next stage starts from the last stage's positions, and nothing pulls back. What should be remembered is carried by the next stage's rules (A1, option 1). The done-condition becomes "starts from the captured layout and keeps a stated relation figure", for example how much of each node's orthogonal order survives (after Misue et al.).
2. **Capture as relations.** The captured layout's orthogonal order and proximity become constraints for the next stage. This needs A1's constraint track.
3. **Keep capture-as-anchor** as the one sanctioned pull, inside schedules only, because the option Mark chose named it.

**A4. Drag, and what "anchored" means.**
Evidence:
- **Two drag behaviours.** The canvas writes the drop point into the node's slot, and the anchor then holds the node there (I6). The board pulls the item back toward its slot (I10).
- **Two definitions of "anchored".** The catalog's is "displacement is temporary and the item returns toward home" (I21, line 64). Sceno's is "relaxation may carry it away; moving an anchored item is correct behaviour" (I14, `score.rs:86-88`).
- **mer3ly.** It offers Anchored (13.0) beside Free as a visitor's choice and plans to consume the board (I35).

1. **A drop is where the node now is, and "anchored" means a seeded home with no return** (recommended). Released, a node belongs to physics from the drop point, on canvas and board alike. The canvas keeps writing the drop into the stored placement, so Restore returns there, and the board stops easing items back. The catalog's Anchored line and its intent row are corrected to sceno's meaning, and mer3ly's Anchored becomes a seeded home.
2. **A drop pins until released**, as `pin_focused` does: an explicit, ensure-class hold. Anchored is as in option 1.
3. **Keep Anchored as a per-item return policy outside physics.** The return is a host-clocked transition, as `ReturnMotion` does for labels, not a seiche term, and it is offered beside Free as mer3ly does.

**A5. Positions that encode data.**
Evidence: the practice board places occurrences from data fields (`encoding.x` and `encoding.y` are `Field`s, `graphshell/web_practice.rs:202-203`) and holds them at pull 120. In the probe, pull 120 keeps ρ 0.965 and 73% of the seed's excess edge length: the physics barely shows (§5). The catalog's Scatter scene reads position as a quantitative value. Collision-dodged dot plots hold the encoded axis and let collision move the other.
1. **The encoded axis is an ensure-class constraint, and physics moves only the free axes** (recommended). With both axes encoded, physics only separates overlaps. This commits to an axis constraint declared from the encoding channel, not the arrangement, and retires the practice board's pull.
2. **Keep a pull for encoding-bound positions only**, declared from the encoding, with the practice board as today: the one place where a return means fidelity.
3. **No physics on encoding-bound positions.** Physics is off for scatter-like recipes.

**A6. The handoffs: picking, stopping, switching.**
Evidence: a pick pauses (I2). Pausing, and the end of a budget, leave bodies where motion left them (I3; "pause must not restore"). Return is the explicit "Restore arrangement" (I4). A switch is a scenotime transition while paused, and only its final slots seed the world (I15). All five items are consistent with F11 (§4).
1. **Keep them as they are** (recommended): a pick pauses, stopping stays, restore is explicit, and a switch is a transition.
2. **A pick keeps the play state.** The transition runs, then physics continues from the landed positions, with no pause on pick.
3. **Stopping returns by a transition.** Pause animates back to the arrangement, so Restore becomes implicit.

**A7. Recipe slots, and whether the two domain bindings are one.**
Evidence: the authored projection definition has no motion slot (`scenograph/src/lib.rs`, lines 240–251), and physics lives in host state (`physics_enabled`; `SavedSceneV1`'s flattened fields). G4 carries the `DynamicsSpec` in `SavedSceneV1`. The physics channels and cartography's disclosures read the same facts (grammar brief, §5.5), and the grammar plan's open question 4 asks whether one registry produces both. Physics also reads facts no arrangement reads (PageRank mass, meaning pairs).
1. **Two slots, one binding** (recommended). A scene recipe names an arrangement and, optionally, a dynamics recipe. One domain binding supplies the disclosed facts both read, and the permitted actions (drag, pin) both honour. G2's registry produces cartography's disclosures and the physics channels from one computation, which settles open question 4.
2. **Dynamics beside the scene recipe.** The dynamics recipe travels next to the scene (in `SavedSceneV1`, as G4 plans) with its own binding, `PhysicsChoice`'s successor, and the scene recipe stays free of motion.
3. **Two slots, two bindings.** As option 1, but with the dynamics binding separate, because physics reads facts that no arrangement does.

**A8. The catalog's §6 force row.**
Evidence: §6 lists "Force and constraint", "Spring, collision, gravity, clusters", "Landed in Seiche" among the arrangement families (catalog, 217), against "Arrangements are positions, not motion". The force-directed arrangements that once produced positions by a converged pass (`ForceDirected`, `BarnesHut`) left with the retired `arrangements` crate (seam doc, Finding 2; physics plan, 265–270). F13 already plans "a pointer from the projection grammar catalog's §6" (grammar plan, 78).
1. **The row leaves the arrangement table, and §6 points to the dynamics grammar** (recommended). If A1 chooses constraints, the arrangement-derived constraints join §6 as constraints on positions.
2. **The row becomes "Settled".** A law's converged result, recorded as positions, is an arrangement; live physics moves to the dynamics grammar.
3. **Keep the row as it is.**

## 9. Sources

Cited from the reference record; nothing was fetched this session. No code was copied from any of them.

- Dwyer, T., Koren, Y., Marriott, K. "IPSep-CoLa: An Incremental Procedure for Separation Constraint Layout of Graphs." *IEEE TVCG* 12(5):821–828, 2006.
- Dwyer, T., Koren, Y. "Dig-CoLa: Directed Graph Layout through Constrained Energy Minimization." *IEEE InfoVis* 2005.
- Dwyer, T. "Scalable, Versatile and Simple Constrained Graph Layout." *Computer Graphics Forum* 28(3) (EuroVis 2009).
- Gansner, E., Koren, Y., North, S. "Graph Drawing by Stress Majorization." *Graph Drawing* 2004.
- Wang, Y. et al. "Revisiting Stress Majorization as a Unified Framework for Interactive Constrained Graph Visualization." *IEEE TVCG* 24(1), 2018.
- He, W., Marriott, K. "Constrained Graph Layout." *Constraints* 3(4), 1998.
- Kamps, T., Kleinz, J., Read, J. "Constraint-Based Spring-Model Algorithm for Graph Layout." *Graph Drawing* 1995.
- Hoffswell, J., Borning, A., Heer, J. "SetCoLa: High-Level Constraints for Graph Layout." *Computer Graphics Forum* 37(3) (EuroVis 2018).
- Misue, K., Eades, P., Lai, W., Sugiyama, K. "Layout Adjustment and the Mental Map." *Journal of Visual Languages and Computing* 6(2):183–210, 1995.
- Friedrich, C., Eades, P. "Graph Drawing in Motion." *Journal of Graph Algorithms and Applications* 6(3), 2002. Animated transitions between layouts, the transfer for §6.4.
- Macklin, M., Müller, M., Chentanez, N. "XPBD: Position-Based Simulation of Compliant Constrained Dynamics." *MIG* 2016.
- Ye, K. et al. "Penrose: From Mathematical Notation to Beautiful Diagrams." *ACM TOG* 39(4), SIGGRAPH 2020.
- Wilkinson, L. "Dot Plots." *The American Statistician* 53(3):276–281, 1999.
- d3-force 3.0.0: positioning forces, collision and the beeswarm idiom, as the grammar brief recorded it (§4.1, §12).

## 10. Not verified here

- **Which reading the 2026-07-23 receipt's frame shows.** `02_spiral_played.png` was written two minutes before the anchor commits and could be either; §5 measures the phenomenon on its own fixture instead.
- **The practice workspace's receipts** for the board at pull 120 (I11) were not read.
- **A ruling on mer3ly's Anchored/Free choice.** None was found in mer3ly's docs; the search was by keyword, so this is an absence in a keyword search, not a proof.
- **A doc claim in the canvas.** `pictograph/strategy.rs:62-64` and `pictograph/../canvas.rs:477-478` say "the host gates its per-frame `project_canvas_strategy` call" on `needs_strategy_recompute`. At the audit base no port in mere calls it: the Graphshell hosts compute slots on an explicit apply or a scene open. Other repositories' hosts were not checked. Noted only; it is outside this brief's subject.
