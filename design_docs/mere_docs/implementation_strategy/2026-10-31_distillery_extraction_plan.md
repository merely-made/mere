# Distillery extraction plan

**Date:** 2026-10-31
**Status (2026-10-31):** plan. Rulings D1 to D12 recorded. Nothing has moved yet. Gate G1
(publishing cambium 0.6.0 and sprigging 0.3.0) waits for Mark.
**Scope:** Distillery, its job and lease layer `mere-mesh`, and the workshop halves of
Alembic and Athanor leave Mere for the ESP repository, which is renamed `distillery`.
Distillery becomes a producer: it keeps its own surfaces and speaks only the published
contract crates. The graph halves stay in Mere as `pandect` modules. Every consumer of
`esp.git` moves to `distillery.git` in the same pass.

## Origin

Mark, 2026-10-31: "extract distillery from mere into ESP, as planned. Remove old
directories. Rename the esp repo distillery." No dated plan recorded the move; the
nearest record is isocosm case 1044 ("esp, Distillery and the other ports being
extracted from mere"). The same session refined stack.1 (house-style D12): extraction
waits for a planned second host, not an arrived one.

## Decisions

- **D1. The move (Mark, 2026-10-31):** "commit and extract distillery from mere into ESP,
  as planned. Remove old directories. Rename the esp repo distillery."

  Links: draws on house-style D12 · summary Distillery leaves Mere for the ESP repository, which becomes `distillery`; donor directories are removed.

- **D2. The back edge (Mark, 2026-10-31):** "Cut first", over Knot-style lockstep.
  Distillery imported eleven Mere crates; the repository it joins stays free of
  `mere.git`, as ESP is today.

  Links: summary Cut Distillery's Mere imports before moving it; the receiving repository takes no mere.git dependency.

- **D3. Contents (Mark, 2026-10-31):** "Port + alembic", over the port alone.

  Links: summary ports/distillery and crates/distillery/alembic move together.

- **D4. Consumers (Mark, 2026-10-31):** "Sweep all six now", over relying on GitHub's
  redirect. Mere, cleromancy, eidetic, knot-editor, mer3ly and turnstone name `esp.git`;
  Cargo treats the old and new URLs as different sources.

  Links: summary Every esp.git consumer moves to distillery.git in this pass, one source per graph.

- **D5. mere-mesh (Mark, 2026-10-31):** "Moves with Distillery", over staying in Mere or
  going to Armillary. It is Distillery's job and lease layer (15,107 lines, used in 15
  Distillery files) and depends on nothing in Mere; Djinn and Graphshell take it from
  `distillery.git`.

  Links: summary mere-mesh moves into the distillery repository.

- **D6. Djinn holds no endpoints (Mark, 2026-10-31):** "I'm not sure djinn should be an
  endpoint like that. Isn't it execution? Not state? It would be like luggage becoming
  host". Then "Moot-style port" named `mere-distillery`; superseded by D7.

  Links: summary Djinn supervises and serves no endpoint; the adapter-port answer is superseded by D7.

- **D7. Distillery is a producer (Mark, 2026-10-31):** "Why don't we just make Distillery
  not an app itself, but a surface producer for mere, right? And then GraphShell is
  actually a useful thing, because then it's like, it is the way, it is the host." Then
  "Producer", over the adapter port. Chronicle, `inference.ask` and the installed
  surface stay in Distillery, which takes the contract crates from crates.io; Mere
  patches them back to its paths.

  Links: amends distillery-extraction D6 · summary Distillery keeps its surfaces as a producer over the published contract crates; no mere-distillery port.

- **D8. Leaf and branch (Mark, 2026-10-31):** "Both must hold", over owning a graph alone
  or hosting alone, after seeing every repository classified. A branch owns a graph (links
  `mere`, `mere-kernel` or `pandect` for its own logic) and hosts others' surfaces; anything
  else is a leaf that produces surfaces, with or without a standalone frame. Refines
  stack.3.

  Links: amends stack.3 · summary An app is a branch only if it owns a graph and hosts others; otherwise it is a producer leaf.

- **D9. Any branch hosts (Mark, 2026-10-31):** "Any branch hosts", over Graphshell as the
  only host. Graphshell is the reference host that loads whatever is installed; Turnstone
  stays the flagship. Refines the 2026-08-22 suite census without reversing it.

  Links: amends stack.3 · summary Every branch can host producers; Graphshell is the reference host, Turnstone the flagship.

- **D10. Order (Mark, 2026-10-31):** "Extract, then survey". The other ports are classified
  against D8 in a dated brief after this extraction.

  Links: summary Finish Distillery first; survey the other ports against D8 after.

- **D11. Djinn is execution only (Mark, 2026-10-31):** "Execution only", over Djinn owning
  the resident's meres. This reopens the 2026-08-22 census line giving Djinn the store
  lifetimes. Recorded as an open fork, not acted on here.

  Links: summary Djinn should hold no meres; who owns store lifetimes is open.

- **D12. Graph halves (Mark, 2026-10-31):** "Modules in pandect", over a slim
  `mere-athanor` left in Mere. Alembic's `memory_levels` becomes `pandect::memory_levels`
  (pandect already depends on it); Athanor's `furnace` becomes a `pandect` module behind a
  feature, which Turnstone's recycle bin takes.

  Links: draws on house-style D2 · summary Graph halves become pandect modules; mere-alembic and mere-athanor leave Mere whole.

## Open forks

- **F1 (D11).** If Djinn holds no meres, which owner takes the resident's store lifetimes?
- **F2, gate G1.** Distillery's installed surface draws with Cambium (`GenetAppRunner`,
  `RunnerSurfaceSession`). crates.io has cambium 0.5.0 and sprigging 0.2.2; Mere has 0.6.0
  and 0.3.0. Publishing them is outside the five contract crates D7 named.

## Phases

### P1. Cut the Mere imports, in Mere

Move `alembic::memory_levels` into `pandect` and `athanor::furnace` into a `pandect`
feature; inline the one `pandect::write_bytes_with_backup` call. Drop the `recall` and
`furnace` features. Point Turnstone's recycle bin at the pandect feature.

*Done when:* `distillery`, `mere-athanor`, `mere-alembic` and `mere-mesh` have no normal
dependency on `mere`, `mere-kernel` or `pandect`; their only Mere-hosted dependencies are
the contract crates (`sceno`, `scenotime`, `chirograph`, `graphshell-endpoint`,
`mere-surface-api`, `scenograph`, `scenomise`, `incipit`, `cambium`); Djinn, Graphshell
and pandect tests pass.

### P2. Receive in the renamed repository

Filter Mere's history to the moved paths and merge it into ESP, laid out as
`crates/{distillery,athanor,alembic,mesh}` plus the probe workspaces. Contract crates come
from crates.io (G1 for cambium and sprigging).

*Done when:* moved sources match the donor at the P1 commit, with recorded manifest
exceptions; the workspace builds and its tests pass standalone, with no `mere.git` source
in its lock; history is preserved.

### P3. Rename and cut over

Rename `merely-made/esp` to `merely-made/distillery`. Mere takes `distillery`, `mere-mesh`,
`mere-athanor` and `mere-alembic` from `distillery.git` at one revision and patches the
contract crates from crates.io to its paths. Sweep the six consumers named in D4. Remove
the donor directories.

*Done when:* no manifest in the six repositories names `esp.git` or a removed path; each
lock resolves one source per package; Mere's touched crates and each consumer's check
pass; changes are committed in each repository.

## Findings

- 2026-10-31: Distillery's Mere imports are concentrated in `chronicle.rs`,
  `chronicle_recipe.rs`, `inference.rs` (the `InferenceAskEndpoint`, lines 1669-2028),
  `surface.rs` and one call in `installed.rs`, all contract crates except that
  `pandect` call. `mere-mesh` is imported as `mesh` across 15 files.
- 2026-10-31: `pandect` depends on `alembic` with `recall`
  (`crates/system/pandect/Cargo.toml:29`); Turnstone depends on `mere-athanor` for its
  recycle bin (`turnstone/Cargo.toml:34`, `src/recycle.rs`).
- 2026-10-31: crates.io holds the local versions of sceno 0.0.6, scenotime 0.0.4,
  chirograph 0.0.2, graphshell-endpoint 0.0.2, mere-surface-api 0.1.0, scenograph 0.0.5,
  scenomise 0.0.6, incipit 0.2.0, hest 0.2.0, workbench 0.1.0. Equal version numbers do
  not prove equal source; P2's standalone build is the check.

## Progress

- 2026-10-31: assessment and rulings D1 to D12; no code moved.
