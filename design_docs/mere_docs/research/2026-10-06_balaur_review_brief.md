# Balaur Review Brief

**Status:** open. The review lane writes its findings into §3.

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

*To be written by the review lane.*
