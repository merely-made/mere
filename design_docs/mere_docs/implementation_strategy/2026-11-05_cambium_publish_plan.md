# Cambium publish plan

**Status:** 2026-11-06: in progress. P0 landed in Mere and every consumer (Woodshed and its
Hocket/Redshank ports, Knot, Cleromancy, Isocosm, Turnstone, Signalman); P1's accesskit removal
landed, and so did its Vello renames (Netrender ad4be200e). Nothing is published yet.

**For:** publishing the Cambium desktop chain to crates.io so that applications outside Mere,
first the Dramatis application (source homes map, ownership cut 1), depend on registry
versions rather than `mere.git` pins. Opened by Mark's choice to publish the app's GUI
dependencies first and fall back to tracking `mere.git` main (D1).
Not here: the Dramatis application move itself (`dramatis/design_docs/`, the app plan after
its move); Genet's and Netrender's own release policy beyond the crates named below.

Receipts go to `archive_docs/receipts/2026-11-05_cambium_publish_plan_receipts.md`.

## Constraints

- Every published crate depends only on crates.io versions; a family that two repositories
  consume keeps one type identity (the cambium family resolves from one source per graph).
- Names are checked on crates.io at the time of the ruling and claimed by a real publish.
- Exact pins (`=0.1.2` and similar) move in every consumer manifest with each bump.
- No upstream contact without Mark's authorization (Netrender's vello notes: "upstream contact
  remains unauthorized").
- Security domains follow standards: nothing here touches crypto.

## Closure

Measured 2026-11-05 by `cargo metadata` over Mere, Genet and Armillary and the crates.io API
(subagent read, spot-checked). About 30 crates: the eight the app needs, their unpublished
dependencies, and roughly seventeen published crates whose manifests drifted since release.

- **New:** `apparatus-core` (Armillary); `genet-render`, `genet-presenter`, `genet-winit`,
  `genet-flow` (Genet); `cambium-rootstock`, `cambium-a11y`, `mandrake`, `cambium-desktop`,
  `persona-picker` (Mere); the renamed vello sparse crates (D5).
- **Republish (drifted):** `cambium` (chartulary ^0.3), `cambium-winit` (registry 0.3.0 wants
  `cambium ^0.3.3`), `buckram`, `livery`, `cadency`, `genet-livery`, `genet-parley`,
  `genet-fontique`, `genet-host-api`, `genet-scripted-dom`, `genet-static-dom`,
  `layout-dom-api`, `taproot`, `netrender`, `netrender_device`, `paint_list_render`.
- **Order:** Armillary → Netrender (with D5's crates) → Genet → Mere.
- Source drift of crates "already published at matching deps" was not diffed.

## Phases

### P0. Rename `mesquite` to `mandrake` (D2, D3)
`crates/cambium/mandrake`, package and lib `mandrake`. The receipt schema id
`mesquite.commands/v1` is a wire name and stays.
- Done when: every Mere consumer checks on all targets (native and wasm32), `mandrake`'s tests
  pass, and Woodshed (root and its `ports/` workspaces), Knot, Cleromancy, Isocosm, Turnstone and
  Murm's `apps/signalman-desktop` take the new name with their Mere pin bump. Knot, Isocosm,
  Cleromancy, Turnstone and Signalman also drop their `[patch.crates-io] accesskit` row, which
  pointed at Mere's removed patch, at the bump that reaches the remap.

### P1. Remove the blockers
- accesskit (D4): `cambium-a11y` remaps through accesskit's public `Node` API (8 vector and 7
  single NodeId properties, text selection, tree root, focus, action targets) with a fixture
  test setting every property and a serde key-set canary; drop `support/patches/accesskit`.
- vello (D5): publish the fork's sparse crates under Netrender names and point Netrender at them.
- iced/egui (D6): move `embedding-egui`/`embedding-iced` out of `cambium-rootstock` into a probe
  crate under `crates/probes`; Graphshell's `caller-owned-toolkit-bench` follows.
- Metadata: `publish = false` lifted where the closure requires it; Genet crates' inherited
  `repository` (servo/servo) corrected.
- Done when: each crate passes `cargo publish --dry-run` against registry-only dependencies
  in order, and the accesskit canary test fails when a property is left unmapped.

### P2. Publish in order
- Done when: every crate in the closure is on crates.io at the planned version, consumers' exact
  pins are updated, and the Dramatis application builds from registry versions alone.

## Decisions

- **D1. Publish the app's GUI dependencies first (Mark, 2026-11-05):** "3, then 2", over
  tracking `mere.git` by rev or main from the start: publish, and track main for anything not
  yet publishable. Reaffirmed as "Full wave now" after the closure was shown at about eight
  crates, and pursued after it grew to about thirty with the blockers below examined.

  Links: summary The Dramatis app's GUI chain goes to crates.io; mere.git main is the interim.

- **D2. `mesquite` becomes `mandrake` (Mark, 2026-11-05):** "Mandrake", over `rhizotron`,
  `herbarium` and `burdock`; `mesquite` is taken on crates.io by an unrelated crate. Rhizotron
  is banked as a pane name. Outcome: new.

  Links: summary The scenario lane publishes as mandrake.

- **D3. Rename everywhere, consumers now (Mark, 2026-11-05):** "Rename everywhere", over a
  package-only rename; and "Now, in this wave" for Woodshed, Knot, Cleromancy and Isocosm,
  confirmed "Rename now regardless" when the wave's size was shown. Outcome: new.

  Links: summary Package, lib and imports take mandrake in Mere and its four consumer repositories now.

- **D4. Drop the accesskit patch (Mark, 2026-11-05):** "Drop patch, public-API remap", over a
  renamed fork and over keeping the patch. Put to Mark on finding every NodeId carrier public
  in accesskit 0.25.1 and no mapping API in 0.25.2. Outcome: new.

  Links: summary cambium-a11y remaps through accesskit's public API with an exhaustiveness canary; the local patch goes.

- **D5. Publish the vello sparse fork under Netrender names (Mark, 2026-11-05):** "Publish
  renamed sparse crates", over registry 0.3.0 with a pre-flight and over authorizing
  upstreaming. Put to Mark on finding Netrender's CPU path calling fork-only `FlattenError`
  refusal (`sessions.rs:345,352,553`, `webgl.rs:341`) and the fork's tolerance fix (`SQRT_TOL`
  0.5 → 1/16). Follows the `netrender-vello` precedent. Outcome: applies that precedent.

  Links: summary The fork's sparse vello crates publish under Netrender names, keeping exact curve refusal and the tolerance fix.

- **D6. Bench embeddings leave rootstock (Mark, 2026-11-05):** "Move to a probe crate", over
  dropping them and over pinning iced 0.14. The iced/egui embeddings serve only Graphshell's
  `caller-owned-toolkit-bench`; iced is an unreleased master rev. Outcome: new.

  Links: summary Toolkit-embedding benches live in a probe crate; published rootstock carries no git dependency.

## Findings

- 2026-11-05: `persona-picker` alone is publishable today: its name is free and `cambium`,
  `dramatis` and `personae` are published.

## Progress

- 2026-11-06: P1's Vello item done (D5). The fork's sparse crates are `netrender-vello-common`,
  `-cpu`, `-gpu`, `-gpu-shaders` (0.3.0) and `netrender-glifo` (0.4.0; Mark, 2026-11-06,
  "netrender-glifo"), with library names unchanged, on `mark-ik/all-vellos` at `7784613` on top
  of `dfb9b53b` (Mark: "Branch tip dfb9b53b"). Netrender `ad4be200e` takes them; its lock is a
  pure rename. A combined `cargo publish --dry-run` packages and verifies all five against
  cargo's temporary registry. Netrender's `vello-all` tests pass except three Classic-path GPU
  tests that fail identically on the parent (`compose_into_01_identity_matches_render`, two
  `cast_slice` alignment panics); no Netrender test triggers `CurveFlattening`, which only the
  fork tests.
- 2026-11-06: P0 done. The rename reached every consumer in the same round that restored the
  family source convention (family brief, "Family source convention", upheld 2026-11-06):
  Woodshed 17ff090d, Knot 40ac979, Cleromancy 4df159b, Turnstone 5be7608c, Murm/Signalman
  1aebf9c1542, Isocosm 26306b02. Each also dropped its accesskit-from-Mere patch row. The
  `mesquite.commands/v1` wire id and Isocosm's `mesquite_byte_scope` receipt key stay.

- 2026-11-05: closure mapped; blockers examined; D1 to D6; P0 rename landed in Mere.
- 2026-11-05: accesskit remap (`cambium-winit-a11y/src/remap.rs`) replaces the patch: 28
  `cambium-a11y` tests pass, including a fixture over all 16 NodeId properties and a canary that
  reads AccessKit's own `Properties` schema; Mere and the standalone `native-id-consumer` probe
  resolve registry accesskit. The probe's dependency also had a stale package name, fixed.
