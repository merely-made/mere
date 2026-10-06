# Balaur Review Brief

**Status:** findings written 2026-10-06 (§3); forks A to E (§3.9) are with Mark.

**Subject:** [balaurengine/balaur](https://github.com/balaurengine/balaur), "A 2D & 3D
node-based game engine, fully deterministic, with scripts that reload in
milliseconds". It is MIT licensed, "Copyright (c) 2026 Sébastien Crozet, Dragos
Daian". Crozet is the Dimforge author of rapier, parry, kiss3d and nexus. Mere
already carries his nexus binning under Apache-2.0
(`crates/conatus/conatus/src/resident/binning`, with its `LICENSES.md` row). The
repository has a `.claude/` folder and a `CLAUDE.md`, so it is partly
AI-assisted. Its licence and named holders are what count for reuse.

## 1. Rulings (2026-10-06)

Mark asked whether Scenograph could take inspiration or a thin capability slice
from balaur, "since it's MIT and all". The coordinator read balaur's README,
`ARCHITECTURE.md` and `LICENSE` and put three questions to him.

- **How to take from balaur.** The options:
  - a read-only review at one pinned commit, recording file references and
    findings, with any port as its own decision carrying a `LICENSES.md` row;
  - going straight into a Scenograph editor plan that cites balaur's
    architecture;
  - inspiration only.

  Mark: **"Read-only review first (Recommended)"**.
- **Whether the games wing hears about it.** Balaur composes kiss3d 0.46, hecs
  0.11, rapier 0.35, egui and taffy, the stack the wing ruled. Options: tell the
  wing session; not now. Mark: **"Tell the wing session (Recommended)"**.
- Recorded alongside: rapier's `enhanced-determinism` was measured as free within
  run spread at 500, 2,000 and 5,000 bodies
  (`Code/testing/mere/rapier-determinism/probe.log`). This is stack seams S4's
  evidence; the rest of S4 is track G8 in the dynamics grammar plan.

## 2. What the review reads for

The six slices named when the question was put:

1. **The editor is a project of the engine itself.** "Scenograph built with
   Cambium", made literal.
2. **A live mirror, with undo on the document rather than on commands.**
   `scene.instantiate(scripts = false)`; Play attaches scripts and unpauses;
   Stop rebuilds the mirror from the document; edits round-trip through encode
   and write.
3. **An inspector generated from the property registry.** A closed set of
   property types, and an undeclared key refused by name.
4. **"A tween is a generated clip: one sampler, two authoring front-ends".** This
   bears on expansion lane L5 (motion).
5. **A per-tick 64-bit digest.** Floats are hashed by bit pattern and labelled by
   stable id. Alongside it come the snapshot ring and JSON Lines record and
   replay. These bear on G8's instruments and expansion lane L4 (projection
   captures).
6. **Prefab instances whose overrides patch rather than replace,** and stable
   u64 ids that survive renames and reparenting.

For each slice the review answers four questions:
- where it lives, by file and line at the pinned commit;
- how tightly it couples to balaur's registry, ECS, kiss3d and egui;
- what Scenograph, Cambium or seiche has today that it would meet;
- whether it is worth taking as a pattern, as a port, or not at all.

## 3. Findings

*Written 2026-10-06 by the review lane, at balaur
`de0df794eee4eea223ed7efd31461044c71999f0`.* Everything was read through
WebFetch, on pinned raw files, tree pages and blob pages. Nothing was cloned,
fetched by cargo, built, run or copied. The verdicts below are the lane's
reading, not rulings. The choices with more than one defensible answer are in
§3.9 for Mark.

**How to read the citations.** Balaur's paths are written `balaur/<path>` so
they are not taken for Mere's. WebFetch's reader miscounts lines in long files.
Three reads of `balaur/editor/scripts/model.rn` put `expand_instances` at three
different lines, and the file's blob header gives 2,010 lines where a raw read
reported 1,459. So a balaur citation names the file, its length from the GitHub
blob header where that was read, and the item; it gives no line numbers. Mere's
line numbers come from `sem` and the files themselves. A port would re-read its
file at the commit.

**Corrections to the premises in §2 and §1.**
- Ids are strings, not u64. `StableId` wraps a `String`, at the end of
  `balaur/crates/balaur_core/src/components.rs` (1,199 lines). Scene files carry
  authored slugs such as `n_ball`. Nodes spawned at run time get
  `<authority>:<counter>` from an allocator on the root
  (`balaur/crates/balaur_core/src/ids.rs`, 135 lines).
- The ring that ARCHITECTURE.md calls `SnapshotRing` is `CheckpointRing` in the
  code (`balaur/crates/balaur_core/src/snapshot.rs`, 737 lines).
- The workspace declares rapier 0.36, not 0.35 (`balaur/Cargo.toml`).
  ARCHITECTURE.md still says glamx is pinned to match rapier 0.35. kiss3d is
  0.46 on a fork (§3.8), and hecs is 0.11.
- ARCHITECTURE.md's "five personas" are five workspaces in
  `balaur/editor/scripts/defs.rn`: Scene, Script, Animation, Physics and UI. Each
  names the docks it shows on the left, the bottom and the right.

### 3.1 The editor is a project of the engine

*Where.* `balaur/editor/` is an ordinary balaur project. It has a short
`project.toml`, 59 Rune scripts under `balaur/editor/scripts/`, and its own
scenes, themes and plugins. The manifest sets the main scene, draws only on
demand, holds the editor to its own checker in strict mode, and sets the
shell's width breakpoints. ARCHITECTURE.md §"The editor is a game" describes
it. `balaur/docs/PLAN-editor-as-scene.md` records the shell's views moving from
immediate `ui::*` calls (1,038, down to 549) into 46 nodes of the editor's main
scene, laid out by taffy.

*How.* `balaur edit <game>` runs the editor project with the game's path in
`engine.args()`. The editor has no private API. It uses bindings any game has:
`fs`, `toml`, `scene.instantiate`, `engine.reload_script`, `require` with
in-place reload, `log.recent`, physics pause and clear, and render's camera and
debug drawing. Saving an editor script hot-reloads the editor itself.
`balaur edit <game> --state "...,shot=out.png"` puts the editor in a named
state offscreen and captures it. `scripts/e2e.sh` runs its self-tests headless.

*Coupling.* Total: Rune scripts over balaur's widget nodes, egui and kiss3d.
The rule transfers; no code does.

*What it meets.* Cambium is already the toolkit a Scenograph editor is built
with. It has workbench tiles (`crates/cambium/workbench`), one command model
(`crates/cambium/cambium/src/command_surface.rs`), the graph canvas, and
rootstock's texture producers for a viewport
(`crates/cambium/cambium-rootstock/src/producer.rs`).
`ports/graphshell/src/projection_editor.rs` is host-neutral editor state: a
draft and a typed `EditorAction` reducer, over workbench tiles. It
"deliberately has no graph, endpoint, or authority handle". mesquite
(`crates/cambium/mesquite`) is our counterpart to `--state` and `e2e.sh`: a
scripted scenario, its captures and one JSON receipt.

*Verdict: a pattern, largely in place.* What carries over is the rule itself.
The editor gets no API a host lacks. It can be put into a named state and
captured by the same lane that tests products. No port.

### 3.2 The live mirror, and undo on the document

*Where.* `balaur/editor/scripts/model.rn` (2,010 lines):
- `load` parses the game's manifest and main scene into a document. It keeps
  the whole parsed scene, so the scene's other tables survive.
- `build_mirror` clears physics and makes an `Edited` child under the root. It
  encodes the authored document, instantiates it with scripts off, and pauses
  physics.
- `attach_game_scripts`.
- `save` patches the file, so comments and formatting survive.
- `doc_value` turns a live component's values back into the document's own
  spelling.

`balaur/editor/scripts/shell.rn` holds `toggle_play` and `release_world`.
`balaur/editor/scripts/history.rn` (128 lines) was read verbatim.

*How.* The document is the only truth, and the mirror is a disposable instance
of it. An edit records history, changes the document's row, writes the live
node, and reads the live node back into the document. Play rebuilds the mirror
fresh, since a mirror posed while editing is not where a recording starts. It
starts a session recording, attaches the game's scripts and hands over to the
game's cameras. Physics and animation are released one frame later, so the
frame the session opens in is not recorded. Stop ends the recording and
rebuilds the mirror from the document. Neither Play nor Stop touches history.

History keeps whole deep copies of the document, the selection and the scene's
asset blocks, never inverse commands. The stated reason is that an inverse per
property would bring back the per-type knowledge the schema-driven inspector
removed. The rest of the mechanism:
- The stack holds 100 frames (`MAX`).
- Every mutator calls `record` before it mutates. A drag that keeps the same key
  within 0.4 s of wall-clock time (`COALESCE`) stays one step. `commit` breaks
  the run when the selection moves or a drag ends.
- `undo` and `redo` swap frames between the past and future stacks. Rebuilding
  the mirror is left to the caller.
- `dirty` compares the stack's position with a saved index, so undoing back to
  the save reads clean. A saved frame that falls off the end of the stack can
  no longer be reached.

*Coupling.* The design stands alone. The code is about 130 lines of Rune over
balaur's TOML document, not worth porting.

*What it meets.* `ProjectionEditor` in `ports/graphshell/src/projection_editor.rs`
holds a `ProjectionDraft`, which is small, `Clone`, `Eq` and serializable. It
reduces typed actions (`reduce`, L265–309) and has no undo. Its preview already
works the way balaur's mirror does. `ProjectionCompiler`
(`crates/cambium/scenes/scenomise/src/projection.rs`, L530–614) compiles from the
draft, the dataset and the host's item sizes as a pure function. So the
preview is rebuilt from the document on every edit.

*Verdict: a pattern, high value for little code.* A history of whole drafts
beside the reducer gives the projection editor undo, redo and a dirty flag,
with no inverse per action. It needs a capped stack, a coalescing key timed by
a clock the host supplies, and the saved position. Where it lives is fork A.
When Scenograph gains a run mode, with seiche playing, balaur's rule carries
over: stopping rebuilds from the document, and running never writes history.

### 3.3 An inspector generated from the property registry

*Where.*
- `balaur/crates/balaur_core/src/components.rs` (1,199 lines). The module doc
  lists the closed type set. It holds `ComponentDef`, `ComponentRegistry` and
  `parse_schema`, and two merge verbs: `add` merges over the schema's defaults,
  and `patch` over the component's current state.
- `balaur/crates/balaur_core/src/components/schema.rs` (551 lines).
  `PROPERTY_TYPES` lists fifteen types: float, int, bool, string, enum, vec2,
  vec3, vec4, color, asset, flags, node, list, map and record. `UNITS` holds
  only degrees. Beside them are `validate_property` and its checkers, and
  `zero_of` with `complete_property`, which give nested specs zero defaults.
- `balaur/crates/balaur_core/src/components/accepts.rs` (43 lines).
  `accept_keys` takes a predicate per component for keys outside the schema,
  used by meta, states, widget and mesh. `refuse_unknown_keys` raises an error
  that names the component and the key, with no suggestion.
- `balaur/editor/scripts/inspector.rn`. It holds `component_sections`,
  `property_row`, and one editor per type: `edit_float`, `edit_int`,
  `edit_bool`, `edit_enum`, `edit_color`, `edit_vector`, `edit_asset`,
  `edit_flags`, `edit_node`, `edit_list`, `edit_map` and `edit_record`, with
  `edit_text` as the fallback.
- `balaur/editor/scripts/pool.rn` reuses row nodes, hiding spares rather than
  freeing them.

*How.* A component registers one TOML schema. That one registration yields its
scene-file key, its script API, its inspector rows and its entry in the
add-component palette. Schemas are validated at boot, and an invalid one panics
naming the component and the property. A scene file, `set_component` and
`patch` all refuse an undeclared key by name. Lists, maps and records nest to
any depth. ARCHITECTURE.md lists `min`, `max`, `range` and `readonly` as spec
fields, and the inspector reads them, but the reads of `schema.rs` found no
check on them. Registration order gives each component its index, up to 128
components, one bit each.

*Coupling.* The schema format and the type set are generic. The registry,
though, is bound to hecs entities and to get and apply hooks on balaur's
`Engine`. The inspector is Rune over balaur's widgets. The ideas stand alone.

*What it meets.* Rust types close most of Scenograph's authored surface:
`Encoding`, `Arrangement`, `Interaction` and the rest, in
`crates/cambium/scenes/scenograph/src/lib.rs`. The exception is
`Arrangement.options`, an open string map by design. Refusal already exists.
`Options` in `crates/cambium/scenes/scenomise/src/catalog.rs` (L380–506)
records every key a family reads through its typed readers: `finite`,
`positive`, `count`, `depth`, `flag`, `choice` and `list`. `finish` refuses
any other key with "does not read this option".

What is missing is the declaration as data. A family's options exist only as
the calls its code makes. So the projection editor can't list them, show their
types, defaults or choices, or generate a row for each one.
`SolverCapability` (`crates/cambium/scenes/scenomise/src/registry.rs`, L56)
advertises a solver's id, name, determinism, disclosures and tags, but not its
options. A registered solver's options are just as invisible.

*Verdict: a pattern.* The lesson from balaur is that one declaration drives both
the refusal and the inspector, so the two can't drift apart. Our seven readers
are already the closed type set. Balaur's fifteen are more than projections
need. How options are declared is fork B. No port.

### 3.4 A tween is a generated clip

*Where.*
- `balaur/crates/balaur_animation/src/sampler.rs` (517 lines) has `sample`,
  a pure function from a clip and a time to a pose. Beside it are `fold` for
  looping and ping-pong, `segment`, `eased`, the Catmull-Rom functions and
  `slerp`.
- `clip.rs` holds the types for clips, tracks, keys, properties, loop modes
  and interpolations, and their TOML parse. A `component/property` address
  stays unresolved until the pose is written.
- `tween.rs` (942 lines) holds `Builder`, `build`, `push_segment`, `offset`,
  `captured` and `current_value`.
- `ease.rs` (359 lines) has eleven transitions in four modes, plus linear.

*How.* A clip is a length, a loop mode and tracks. A track is a target path, a
property, an interpolation (step, linear or Catmull-Rom) and keys. Each key has
an optional ease leading into it. Rotations are authored as euler angles and
sampled as quaternions along the short arc.

A tween is a list of steps: `to`, `by`, `from`, `target`, `interval`, `call`
and `parallel`, each with a duration and an ease. When the tween starts,
`build` turns it into an ordinary clip:
- `from` comes from an earlier step on the same track, or else from the node's
  current value;
- `by` is added to it, composed as a quaternion for a rotation;
- each step writes a start key and an eased end key;
- parallel steps share their group's start.

The same sampler then plays the clip, and there is no second interpolation
path. Because a tween is data, it serializes and hot-reloads, and the editor can
author one.

*Coupling.* The sampler and the easing nearly stand alone. The sampler leans on
glamx, libm, toml and the clip types, and `ease.rs` imports only anyhow. The
clip parse touches `balaur_script::Value` for the arguments of method tracks.
`tween.rs` is bound to `Engine`, hecs, the component registry and Rune values.

*What it meets.* L5 has partly landed since the expansion brief.
`crates/cambium/scenes/scenotime/src/transition.rs` derives a
`TransitionSchedule` from a validated `SceneDiff` and a `TransitionSpec`. It
stages windows for exits, updates and entries, staggered in stable instance
order. Its `sample_at(elapsed_ms)` is pure, and the host supplies the clock.
So scenotime already has the one sampler, for motion driven by diffs. Set
beside balaur, it lacks four things:
- a second front-end, since authored motion (keyed clips, or tween steps) has
  no form, and a consumer wanting a scripted reveal must build a diff;
- easing beyond `Linear` and a smoothstep `EaseInOut`;
- timing for each key rather than for each change class;
- rotation along the short arc. `TransitionValue::interpolate` lerps `rotate`
  in radians, so a turn from just under 2π to just over 0 goes the long way
  round, which balaur's quaternion path exists to prevent.

*Verdict: a pattern.* When L5's second front-end arrives, authored motion should
compile to the schedule the sampler already plays, not to a second
interpolator. The easing set is fork C, and the rotation path fork D. `ease.rs`
is the most portable file in the review. Its curves are public formulas,
though, so a port is not recommended.

### 3.5 The per-tick digest, the snapshot ring, and record and replay

*Where.*
- `balaur/crates/balaur_core/src/digest.rs` (382 lines): `Hasher`, `Entry`,
  `DigestRegistry`, `entries`, `digest`, `fold`, `first_divergence`,
  `node_label` and `hash_value`.
- `balaur/crates/balaur_core/src/snapshot.rs` (737 lines): `SnapshotRegistry`,
  `Checkpoint`, `capture`, `restore`, `CheckpointRing` and
  `build_core_sources`, and the node functions `save_nodes`, `load_nodes`,
  `respawn` and `free_spawned_since`.
- `balaur/crates/balaur_core/src/replay.rs` (1,091 lines): `Header`, `Frame`,
  `Trailer`, `ExternalIo`, `Recorder`, `ReplayPlayer` and `Divergence`.
- `balaur/crates/balaur_core/src/rollback.rs` holds `Session`, and
  `balaur/crates/balaur_core/src/rng.rs` holds the one `Pcg32`.

The account is in `balaur/docs/DETERMINISM.md`. The tests are
`balaur/crates/balaur_physics/tests/suite/determinism.rs` and a threads test.
CI and the lints are `balaur/scripts/determinism_trace.sh` and
`balaur/scripts/house_lints.py`. §3.8 gives the mechanisms byte by byte for
the games wing.

*How.* Every tick folds to one 64-bit number. Floats enter by their bit
pattern, so a drift of one ulp registers. The state is hashed in labelled
slices, and the labels are stable ids rather than paths. So two machines give a
node the same name in a divergence report, even after a reparent. The walk
follows the scene tree rather than sorting by id, because a reparent is itself
state worth catching. Plugins add what components don't report: physics adds
velocity and sleep state.

A recording stores inputs, not state, one JSON line per tick. Verify mode feeds
the inputs back and stops at the first digest that doesn't match.
`--entries-at <tick>` dumps the labelled slices, to diff across machines.
Rollback needs real state, so each subsystem registers a snapshot source.
Restore rebuilds the node set first.

*Coupling.* The hasher, the fold, the entry labels and `first_divergence` are
about a hundred lines of generic logic. The walk is bound to hecs, the scene
tree and the registry. Snapshot, replay and rollback are bound to `Engine`, the
stages and the script host.

*What it meets.* G8 in
`design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`,
and S4 in `design_docs/mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md`.
- The `seiche-repeat` probe (`Code/testing/mere/seiche-repeat`) already uses
  balaur's fold, FNV-1a over `to_bits` with positions sorted by key, but takes
  one fingerprint at the end of 300 ticks.
- seiche's `LayoutSnapshot` (`crates/conatus/seiche/src/view.rs`, L65) holds
  only positions, for the host. It cannot be restored.
- seiche runs two generators: xorshift32 for emitter jitter
  (`crates/conatus/seiche/src/emitter.rs`) and SplitMix64 for the laws
  (`crates/conatus/seiche/src/laws/mod.rs`).
- seiche is on rapier2d 0.33, with neither `enhanced-determinism` nor
  `parallel` on (`crates/conatus/seiche/Cargo.toml`).
- On the Scenograph side (L4), scenotime's `SceneSnapshot`
  (`crates/cambium/scenes/scenotime/src/snapshot.rs`) crosses a wire with its
  tombstones and revisions. Nothing stores one.

*What G8 lacks that balaur has.*
1. **A per-tick trace.** G8 takes one fingerprint at the end. A line per tick
   finds the first tick that diverged.
2. **Labelled slices and the first divergence,** per body by `NodeKey` and per
   term, so a failure names the body and the term.
3. **Velocity and sleep state in the fold.** A fold over positions misses drift
   until it shows up in a position.
4. **Record and replay of external input** (drags, pins, dataset changes), with
   a verify mode.
5. **A checkpoint that can be restored.** It would hold velocities, rapier's
   state, the generators' states, and law state such as Kuramoto phases and
   emitter accumulators. Rewinding or scrubbing a live layout needs one.
6. **One owned generator,** whose state goes into the snapshot and into the
   recording's header. G8's "seeded" declaration says that a term is seeded,
   not where its seed state lives.
7. **A test that thread counts agree,** one solver thread against N. It matters
   only if rapier's `parallel` goes on.
8. **A lint** against bare transcendental calls and iteration in hash order. G8
   routes math through `libm`, with nothing to keep it that way. The probe has
   already found three sums over a randomized map.
9. **A cross-platform trace diff on every push.** G8 asks for one match, on
   Windows and on macOS or Linux. Balaur's CI diffs on every push, and its
   arm64 macOS runner catches FMA contraction.

Going the other way, G8's per-term determinism declarations, refused at compile
time when missing, have no counterpart in balaur.

*Verdict: a pattern, for G8 and for L4.* Items 1, 2, 3, 6 and 8 are cheap. They
go to Mark as amendments to G8 (fork E). For L4, the record format is the
pattern to take:
- a header: a format version, the seed, the source revision, and a fingerprint
  of the spec;
- one line per revision, carrying its `SceneDiff`;
- a trailer, whose absence means the session never closed;
- sources registered by name, with readers skipping unknown ones, so old
  captures outlive new subsystems.

Porting the hasher isn't worth a licence row. FNV-1a is public, and the probe
already has it.

### 3.6 Prefab overrides that patch, and stable ids

*Where.*
- ARCHITECTURE.md §"Prefabs".
- On the editor side, `balaur/editor/scripts/model.rn`: `expand_instances`,
  `expand_one`, `absorb`, `record_override`, `differing`, `refuse_structure`,
  `fresh_id` and `INSTANCE_DEPTH`.
- On the engine side, `patch` and `StableId` in
  `balaur/crates/balaur_core/src/components.rs`, and
  `balaur/crates/balaur_core/src/ids.rs`.

The engine's run-time instancing is not in `scene.rs` or `scene_api.rs`, and
this lane did not find where it lives.

*How.* A node with `instance = "scenes/crate.toml"` becomes that prefab's root.
The prefab's children become its children, with ids prefixed by the instance's
id (`n_crate_b/n_lid`), nesting with the instances. An instance's `overrides`
table is keyed by path from the instance node. Overrides are patched over each
component's current state, not over the schema's defaults, so overriding one
property keeps the rest.

The editor writes only what differs from the prefab's base. `differing`
recurses into tables, and drops an override equal to the base and any table
left empty. Structural edits inside an instance are refused: adding, renaming,
deleting, reordering and instancing. The message points at the prefab's file.
A prefab that contains itself is refused at load, and the error names the
cycle.

Authored ids are slugs of the node's name, prefixed `n_`, with `_2` or `_3`
appended if the slug is taken. Run-time ids come from an allocator on the root
and are snapshotted, so a re-simulation mints the same ids. ARCHITECTURE.md
§"What determinism is still missing" records a gap. A run-time
`scene.instantiate` reuses the file's ids, so two copies collide. The fix is a
minted prefix for each instance. It is held back because the editor's mirror
addresses nodes by those ids.

*Coupling.* `patch` is bound to the registry's get and apply hooks, and
override expansion is editor Rune over TOML. The ideas stand alone.

*What it meets.* Scenograph's `ProjectionVariant`, applied by
`AuthoredProjectionDefinition::arrangement_with_variant`
(`crates/cambium/scenes/scenograph/src/lib.rs`, L671–696), already patches
rather than replaces, one level deep. A variant names its definition and
carries sparse arrangement options, inserted over the base. A variant for
another definition is refused.

On identity, sceno keeps `SourceRef` apart from `InstanceId`
(`crates/cambium/scenes/sceno/src/scene.rs`). `SourceRef` is the source's own
stable id, which the engine never interprets, and `InstanceId` is an index.
scenotime never reuses an index within a `SceneEpoch`, and serializes
tombstones (`crates/cambium/scenes/scenotime/src/ids.rs`). That covers what
balaur's prefixes do. Two instances of one source get distinct instance ids,
and the source id survives a rename or reparent because it belongs to the
source. It also avoids balaur's collision gap.

*Verdict: a pattern for overrides, nothing for ids.* Two rules carry over. When
the projection editor gains variant editing, it should write only what differs
from the base and drop an option equal to it. If variants grow past arrangement
options into encoding or appearance, they should patch field by field rather
than replace. Our identity scheme already fits our case better than balaur's
would.

### 3.7 Other bearing on Scenograph as a Cambium editor

- **The command palette** (`balaur/editor/scripts/palette.rn`). One command list
  feeds both the palette and the shortcuts. Each command carries a label, an
  icon, a shortcut and tags. Commands come from built-ins, workspaces, a plugin
  registry and the scene's own data: presets, components and scene files.
  Matching is a substring search over the label and the tags, with no
  ranking. Cambium's `command_surface.rs` is already one command model,
  rendered as a palette, a picker or a context menu, and it adds reasons for
  disabled commands. The only things worth taking are balaur's tags and its
  commands drawn from the document.
- **Docks and workspaces** (`balaur/editor/scripts/defs.rn`). A workspace is a
  named layout of docks on the left, the bottom and the right. The projection
  editor's workbench tiles already split, stack and tear out, so a workspace
  would be a named tile layout.
- **The timeline** (`balaur/editor/scripts/anim.rn`). It scrubs the node's
  actual player, so the preview is what ships. "Save as file" and "Make inline"
  produce byte-identical documents, so each exactly undoes the other. We
  already have the first rule: the preview compiles through the host's
  `ProjectionCompiler`.
- **Drawing on demand.** The editor draws a frame only when input, a log line,
  a changed file or a scheduled repaint asks for one.
- **Accessibility.** Nothing read here publishes an accessibility tree. Ours
  does, through rootstock's `TextureProducer::semantics` and producer actions,
  so here the lesson runs the other way.

### 3.8 For the games wing

Added at the coordinator's request, for the Isometric game engine
architecture session. It describes balaur only and doesn't judge the wing's
rulings. Isometry's repository was not read.

**Determinism.**
- *The digest* (`balaur/crates/balaur_core/src/digest.rs`). FNV-1a, 64-bit,
  with the standard offset and prime. `hash_value` writes one tag byte per
  value, then:
  - integers as u64, little-endian;
  - floats by their bit pattern, little-endian;
  - booleans as one byte;
  - strings as UTF-8 bytes ending in NUL;
  - arrays element by element, with no length prefix;
  - tables with their keys sorted.

  `entries` walks the scene tree in pre-order. It skips subtrees tagged `local`
  and the debugger's container. Each node gives its slices in a fixed order:
  appearance, transform, tags, then its components in registry-index order.
  Plugin sources follow, labelled `<source>/<label>`. A slice is labelled by
  the node's `StableId`, or by its path where it has none, followed by the
  aspect. `fold` hashes each label with a NUL, then the slice's u64,
  little-endian. `first_divergence` names the first label whose slices differ,
  or a slice missing on one side.

  `balaur run <game> --fixed-tick --trace-digest <file>` writes one
  `<tick> <digest>` line per tick. CI diffs those traces for five examples on
  Linux, macOS arm64 and Windows.
- *The snapshot ring* (`balaur/crates/balaur_core/src/snapshot.rs`).
  - A source is a named pair of closures. One saves a subsystem's state to
    JSON, and the other loads it back.
  - A `Checkpoint` is one JSON map with an entry for each source, encoded as
    JSON bytes.
  - `CheckpointRing` holds pairs of tick and checkpoint, up to a capacity of at
    least 1. A push for a tick it already holds replaces that entry, because
    rollback re-runs stored ticks. A full ring drops its oldest entry.
  - Core's sources are nodes, transforms, appearance, tags, pause state, the
    clock, timers, script state and the generator. Physics and animation
    register their own.
  - The nodes source registers first. So restore frees nodes spawned since the
    checkpoint, and respawns the ones freed before it, before any other source
    writes.
  - Components are deliberately not snapshotted, since re-adding a body would
    rebuild it and lose its velocity.
- *Record and replay* (`balaur/crates/balaur_core/src/replay.rs`). JSON Lines,
  format 3, flushed every frame.
  - The header gives the project, the generator's state, the clock's origin, a
    fingerprint of the scripts, the start time, the tick rate, and any setup
    state.
  - Each tick's line gives the tick, `dt` as f32 bits, a map of each source's
    input that omits empty sources, a frozen flag, an optional digest, and up
    to 2,000 timeline events.
  - The trailer gives why the run ended, the last tick and a digest. A file
    without one means the run crashed.
  - There are twelve input sources and three setup sources. Restore runs as
    core's first system, in `First`, and sends recorded network events down
    the same channels the workers use. `ExternalIo` lets outbound work start
    only outside playback and resimulation.
  - `balaur replay <file> --verify` stops at the first digest that doesn't
    match, with a non-zero exit. `--entries-at <tick>` prints the labelled
    slices.
  - The editor records with one digest, at the end, unless the verify toggle
    in its Session dock is on.
- *Rollback* (`balaur/crates/balaur_core/src/rollback.rs`, with `netsession.rs`
  and `transport.rs`). A `Session` holds the ring, and journals of the inputs
  that arrived, the inputs used, and each tick's digest. A missing input is
  predicted by repeating the player's last one. A late input that disagrees
  restores that tick's checkpoint and re-runs forward, with `is_resimulating`
  set. `stale_inputs` counts inputs older than the ring. Each datagram carries
  the player's last twelve ticks of input, and digests are sent reliably.
- *The generator and the math* (`balaur/crates/balaur_core/src/rng.rs`).
  - The engine owns one PCG32 with seed 0. Its u64 state goes into snapshots and
    into the recording's header.
  - glamx is built with `libm` and `scalar-math`, and rapier with
    `enhanced-determinism`.
  - The Rune fork (`balaurengine/rune`, branch `deterministic-pow`) puts `powf`
    and `powi` on libm.
  - ARCHITECTURE.md says the fork keeps objects and maps in insertion order,
    hashed with a fixed seed. But `DETERMINISM.md` still tells authors that
    `#{}` iterates in hash order. The two disagree, and the fork's code was not
    read.
  - `house_lints.py` rejects bare transcendental calls and bare channels.

**ECS and scheduler.**
- *Storage.* hecs 0.11. A node is an entity with `Name`, `Parent`, `Children`
  and a local `Transform`, and global transforms come from a propagation
  system.

  `Engine` (`balaur/crates/balaur_core/src/engine.rs`) is shared through an
  `Rc`. It holds:
  - the hecs `World`, in a `RefCell`;
  - the resources, in a type map behind a `RefCell`;
  - a `RefCell<Vec<Command>>` of deferred structural changes;
  - the tick, as a `Cell<u64>`.

  By its own doc it is single-threaded by design. Parallelism happens inside a
  system, never across systems. Rapier's solver threads through rayon, by
  default on one fewer than the machine's cores and at most eight. A test checks
  that one thread and eight give the same digest. The component registry allows
  128 components, one bit each in `Attached`, and each component's index is its
  place in registration order.
- *Stages.* `Stage` in `balaur/crates/balaur_core/src/app.rs` runs these in
  order every frame:
  - `First`: input and OS events, with replay's restore before anything else;
  - `PreUpdate`: the hot-reload pump;
  - `Update`: scripts' `update`, and gameplay and presentation systems. Events
    are delivered at its start, ordered first by emission, then by
    subscription;
  - `FixedUpdate`: scripts' `fixed_update`, then physics, because core
    registers the script callbacks first;
  - `PostUpdate`: audio and simulation reactions;
  - `SceneSync`: transform propagation, and interpolation for nodes that opt
    in, for rendering only;
  - `Render`;
  - `Last`: deferred frees.

  A system is a closure, taking the `&Engine` and a `dt`, added to one stage.
  Systems run one at a time in the order they were added, with no labels and
  no before or after constraints.
- *The accumulator.* There is one per `App`.
  - The defaults are 60 Hz (a `fixed_dt` of 1/60) and at most 4 substeps,
    scaled with the tick rate.
  - The accumulator is capped at the step times the substep cap times the time
    scale. It is then drained floor-and-subtract, one whole step at a time.
  - What remains gives `SceneSync` its interpolation alpha.
  - `--fixed-tick` pins the frame rate for headless runs, and a per-app
    `set_fixed_dt` overrides the project's rate.
- *Replacing the frame.* `App::advance` hands a live frame to an installed
  `FrameDriver` in place of the default tick. The multiplayer plugin's driver
  runs the ticks owed through `NetSession::advance`. It waits when a peer lags,
  and skips `Render` on re-runs.
- *Outside the accumulator.* Animation keeps its own accumulator at `fixed_dt`.
  It runs after the script tick but before `FixedUpdate`. A game's pause works
  per subtree, through process ticks, while a debugger freeze holds the whole
  stage. Timings only observe, and are never recorded or hashed.

**kiss3d.**
- *How it's carried.* As a `[patch.crates-io]` git dependency, neither vendored
  nor a path, on `github.com/Ughuuu/kiss3d`, branch `balaur-wrapped-surface`.
  - The manifest's comment says that branch is Ughuuu/kiss3d#1, to be merged
    into `balaur-hooks`, the branch ARCHITECTURE.md names.
  - It is locked at commit `4522a762e83f5d642d0883c92dc149c52d03569c`. That
    commit exists: "Built-in shaders link to one declaration order, so two
    builds match", a change to `build.rs`.
  - The same `[patch]` table carries rune, cosmic-text and the whole egui
    family as forks.
- *Size.* GitHub's compare header gives 82 commits and 125 changed files over
  `v0.46.0`. The diff itself would not render, so no per-file diff was read.
  The branch also carries upstream commits made after v0.46.0, such as
  sebcrozet's egui modifier-key fixes and "Mobile fixes (#408)". One commit is
  by Dragos Daian. The fork's other commits are under the Ughuuu account.
- *What changed,* grouped from the commit titles.
  - **The original hooks** (Sep 4–5).
    - On the web, the canvas is presented opaque.
    - A screen buffer is copied mid-pass for 2D materials.
    - IME events, and UIKit's safe area.
    - A canvas the page supplies keeps the page's scrolling.
    - ⌘ chords work on Apple browsers.
    - egui reuses its last pass on a frame that draws no UI.
  - **Platform and window** (Sep 6–26).
    - Per-vertex colours in `mesh3d`, and launch URLs.
    - Every key with a W3C code reaches the app, and Back and Forward are their
      own buttons.
    - A window can sleep until an event, and another thread can wake it.
    - Focus and visibility events, and app lifecycle and low-memory events.
  - **Rendering settings** (Sep 13–21).
    - Pixels per point, zoom and text scale.
    - SSAO fixes, and optional bloom and post-processing.
    - wesl 0.5.
    - A software adapter for rendering without a surface.
    - Sprite batching, and a public UV rectangle.
  - **The wrapped-surface pass** (Oct 2–3, `balaur/docs/PLAN-wrapped-surface.md`).
    - Skins, the 9-slice mesh, the transparency test and the 2D blend state are
      public.
    - Every shadow setting is exposed. A masked caster's shadow keeps only its
      opaque texels.
    - Cameras take an orthographic height. A stereo camera draws each eye into
      its own half, and its eyes converge on their target.
    - An object can draw over everything, or stay out of the shadows others
      cast on it.
    - A 2D surface culls back faces when asked. Rectangles and tilemaps wind
      counter-clockwise like every other shape.
    - Bloom levels, the cluster grid and a mirror's size are settings. A mesh
      blends up to 256 morph targets.
    - A reflection probe can turn its box and pick its capture planes and
      layers.
    - The sky can light the scene apart from the sky that is drawn.
    - A lit 2D scene takes 64 lights, including directional ones.
    - 2D GI takes segment occluders and marches in field pixels.
    - Offscreen depth can always be sampled, and offscreen surfaces draw at the
      sample count they ask for.
    - Resources are released when the last window drops.
    - The Sobel pass works in the film chain.
    - A translucent instance goes to the transparent pass.
  - **Builds** (Oct 4–5).
    - Built-in shaders are linked when kiss3d compiles, run on GLES, and are
      declared in a fixed order so two builds match.
    - Text, default fonts and image formats are features.
    - A multisampled depth buffer is a texture only while an effect reads it.
    - `set_ime_allowed` no longer raises the Android keyboard.
- *Files touched, as the plan names them.* `PLAN-wrapped-surface.md` names
  `builtin/shadow.rs`, `window/window.rs`, `scene/object3d.rs`,
  `scene/object2d.rs`, `builtin/lit_material2d.rs`, `hdr.rs`,
  `renderer/reflection_probe.rs`, `post_processing/`, `camera/`,
  `builtin/clustered.rs`, `procedural/` and `scene/sprite.rs`. That list comes
  from the plan, not from a verified diff.

### 3.9 Verdicts and forks

| Slice | Verdict | Port? |
|---|---|---|
| 1. Editor as a project | A pattern, largely in place in Cambium and the projection editor | No |
| 2. Mirror and document undo | A pattern: whole-draft history beside `ProjectionEditor` (fork A) | No |
| 3. Inspector from the registry | A pattern: declared option schemas, one declaration for refusal and for rows (fork B) | No |
| 4. Tween as a generated clip | A pattern: authored motion compiles to scenotime's one schedule (forks C, D) | No; `ease.rs` is the only candidate |
| 5. Digest, ring, replay | A pattern, for G8's instruments and L4's record format (fork E) | No |
| 6. Prefab overrides, ids | Overrides: write only the differences. Ids: nothing | No |

No port is recommended. If Mark wants one anyway, the candidate is
`balaur/crates/balaur_animation/src/ease.rs`. It is 359 lines, imports only
anyhow, and is one self-contained module to adapt. Its tests would check that
every curve's endpoints are exact. Its `LICENSES.md` row would follow the
nexus row:
- the port's path;
- "MIT";
- "balaurengine/balaur at `de0df79`, by Sébastien Crozet, Dragos Daian";
- balaur's `LICENSE` in-tree, with a port-and-change notice in the module.

**Forks for Mark,** each with its recommendation first.
- **A. Where whole-document history lives.**
  1. A generic history in Cambium, over any cloneable document, timed by a clock
     the host supplies. The projection editor uses it first, and any later
     Cambium editor after it. *(Recommended: consolidates into the stack, and
     no editor builds its own.)*
  2. Inside Graphshell's `projection_editor.rs` only.
  3. In scenograph, beside the draft.
- **B. How arrangement options are declared.**
  1. Each catalog family and each `SolverCapability` declares its options as
     data, in a closed type set matching today's seven readers. `Options`
     checks against that declaration, and the editor builds its rows from it.
     *(Recommended: one declaration for refusal and for rows, so the two cannot
     drift.)*
  2. Keep refusal by tracking reads, and add a separate descriptor for the
     editor only.
  3. Leave options undeclared, and have the editor show free keys and values.
- **C. Easing, when L5's authored front-end arrives.**
  1. Write our own named set from the public formulas. *(Recommended: small, and
     no licence row.)*
  2. Port `ease.rs` with an MIT row.
  3. Keep linear and smoothstep.
- **D. Rotation in scenotime's transitions.**
  1. A rotation mode per `TransitionSpec`, defaulting to the short arc.
     *(Recommended: a diff between two layouts almost always means the short
     turn, and a consumer that wants a full turn can still say so.)*
  2. Always the short arc.
  3. Keep the literal lerp.
- **E. G8's instruments.**
  1. Amend G8 now with:
     - a per-tick trace with labelled slices and the first divergence;
     - velocity in the fold;
     - one owned generator state per world;
     - a lint.

     *(Recommended: today's five different fingerprints say only that
     something differs. These instruments say which tick, which body and which
     term.)*
  2. The same, plus record and replay of input and a restorable checkpoint, as
     a new track.
  3. Leave G8 as ruled.
