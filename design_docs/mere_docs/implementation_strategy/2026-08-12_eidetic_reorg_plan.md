# Eidetic Reorg Plan

**Date**: 2026-08-12
**Status (2026-10-06):** landed 2026-08-12 in `8595cd38`, authorized by Mark
the same day ("you can reorg eidetic"). `persistence` is
`esp::embed::persistence` behind esp's `persistence` feature, `field_bridge`
and `canvas_search` are in `crates/canvas/pictograph/src/canvas/`, and no
`mere-embed` package remains. Open: done-condition 4's census row (the
[leverage census](../../2026-08-10_leverage_census_brief.md) still reads
`mere-embed` "Keep; wires in as W4"). E-R4's fetchers have since folded into
eidetic-core (`254b23b6`), and graphshell-web has had its own workspace since
`74b55236`; both are noted below.

**Related**:
[search_surface_wiring_plan](2026-08-12_search_surface_wiring_plan.md) (W4
is the consumer of both moves),
[leverage census](../../2026-08-10_leverage_census_brief.md) (§2 rows this
plan resolves), the esp consolidation plan (whose split left the glue
behind).

## The finding this executes

`mere-embed` ("Mere's statistical-intelligence glue over esp::embed") holds
three live modules and has zero consumers. Its own description names the
two halves: eidetic persistence, and the quint field-algebra / canvas-search
bridge. Neither half's natural consumer imports the crate, and the crate
was **never published** (verified against the registry 2026-08-12), so it
can dissolve without a shim — cheaper than the vates/sibylla retirement.

**Two corrections, 2026-08-12, both found by reading manifests during
execution.** They are recorded in the order they happened because the second
undoes the first, and the pair is the lesson.

1. The draft sent `persistence` into eidetic-core, reasoning from eidetic's
   lib.rs naming "vector indices" as one of its own-schema lanes. Reading
   esp's manifest looked like it forbade that — `mere-eidetic` and
   `muniment` appeared as esp dependencies, which would make an
   eidetic→esp edge a cycle.
2. **That reading was wrong**: both are `[dev-dependencies]`. esp's *library*
   has no eidetic edge at all (its only other mentions are doc comments).
   So there was never a cycle, and either direction was open.

The home chosen on the accurate facts is **esp, behind a new `persistence`
feature**, with `eidetic` as an optional dependency. Reasons, now that both
directions were actually available: the module persists esp's own type, so
it belongs beside it; making the storage substrate name the ML seam's types
would invert the layering; and gating it keeps esp's promise that the default
tree is serde-only and portable, which an unconditional eidetic edge would
have quietly broken.

**Census correction.** `mere-embed` was not the true zero the
[leverage census](../../2026-08-10_leverage_census_brief.md) reported.
`eidetic-search` dev-depends on it (`features = ["bert", "bert-wgpu"]`) for
the `eidetic-recall` example — the same bin that produced W1's receipt. The
census's reverse map filtered `kind != dev`, so example- and test-only
consumers were invisible to it. Every "zero consumers" row in that brief
carries this caveat; re-run it including dev edges before acting on another
one.

## Moves

- **E-R1 — `embed::persistence` → `esp::embed::persistence`**, behind the
  new `persistence` feature (revised; see the corrections above). Save/load
  a `VectorIndex` through eidetic's typed-payload API, beside the index it
  persists. Deliberately NOT eidetic-search: that crate's principle is
  zero engine dependencies (`fuse()` takes both rankings from the caller),
  and it keeps it.
- **E-R2 — `embed::field_bridge` + `embed::canvas_search` →
  `crates/canvas/pictograph/src/canvas`**, behind a feature if canvas does not already
  carry quint. Their coupling is quint field algebra projected over the
  canvas — canvas-cluster code that was only ever parked in intel.
- **E-R3 — delete `mere-embed`.** Remove the crate and its workspace
  entries; sweep its doc references (the search plan's W4 paths update in
  the same commit). No published version exists, so no compatibility shim
  and no registry tombstone. Its one real consumer, `eidetic-search`'s
  `eidetic-recall` example, repoints to `esp` (same features plus
  `persistence`) and its `embed::` paths become `esp::embed::`.
- **E-R4 — the fetchers stay crates.** `eidetic-https-fetcher` and
  `eidetic-iroh-fetcher` keep real dependency walls (reqwest; iroh) that
  must not enter eidetic-core; their wiring destinations are already named
  (gazette feed pipeline, mesh blob lane).

  **Corrected 2026-10-06 (S14 pass):** the fetchers are no longer crates.
  `254b23b6` (2026-09-23) folded them into eidetic-core as the features
  `https-fetcher` and `iroh-fetcher` (`crates/eidetic/eidetic-core/Cargo.toml`,
  lines 59-60; `crates/eidetic/eidetic-core/src/https_fetcher.rs` and
  `crates/eidetic/eidetic-core/src/iroh_fetcher.rs`), so the dependency walls
  are now feature gates.

Non-moves, stated so the reorg has edges: muniment, codicil, chartulary,
scholia, tulpa, eidetic-fjall, and eidetic-search all stay put. The family
directory is coherent; the reorg is the dissolution of one orphan crate
into the two homes its halves always had.

## Progress

- **2026-08-12 — executed.** `esp::embed::persistence` behind the new
  `persistence` feature (6 moved tests green inside esp's 68);
  `field_bridge` + `canvas_search` in `mere-canvas` with `esp` added as a
  default-features dep; `mere-embed` deleted with its two workspace
  entries; `eidetic-search`'s example repointed. One environmental blocker
  surfaced on the way and was fixed rather than worked around: the
  workspace check failed with *"failed to find `genet-render-host` in path
  source"*, which reads like a network error and is not — mere's gitignored
  `.cargo/config.toml` redirects the genet git source to path sources, and
  a redirect table must carry **every** package the graph pulls from that
  source. `ports/graphshell/web` git-deps `genet-render-host`, which had no
  entry. Added.
- **2026-08-12 — two pre-existing workspace breakages found, one fixed.**
  With the patch entry in place the workspace check reached
  `ports/graphshell` and failed: `Array<u8, …>: LowerHex is not satisfied`
  at `transfer.rs`. **Attribution settled by running the same check with
  this reorg stashed: it fails identically.** Not caused or exposed by this
  work — my first note here claimed the reorg's feature unification switched
  the `sha2` feature on, and a baseline run disproved it. The cause is
  residue from the 2026-08-10 crypto-generation unification: on the digest
  0.11 row a digest is a `hybrid_array::Array`, which dropped the `LowerHex`
  impl `generic_array::GenericArray` had, so `format!("{:x}",
  Sha256::digest(…))` no longer compiles. The crate's own test had already
  been rewritten to the byte-iteration form, which is the tell. Fixed here
  anyway with a `hex_digest` helper at the two production sites, since it is
  two lines and the tree should not stay red.
- **The second was flagged, then fixed — the workspace has a green gate
  again (2026-08-12).** `ports/graphshell/web` is a `cdylib` of wasm-only
  code (`muniment::IndexedDbBackend`, `wgpu::SurfaceTarget::Canvas`) and is
  a plain workspace member, so `cargo check --workspace` compiled it for the
  native host, where neither symbol exists. That is why nobody ran the
  workspace check here, which is in turn why the graphshell hole survived
  two days.

  **Fixed with a crate-level `#![cfg(target_arch = "wasm32")]`** on its lib
  root (`ports/graphshell/src/web.rs`), so it compiles to nothing off wasm.
  Chosen over moving it to `exclude`: an excluded crate becomes its own
  workspace root and needs a second `Cargo.lock` and a hand-copied
  `[patch]` table (the arrangement that makes signalman-desktop fragile),
  whereas a gated member keeps sharing both. Its doc comment carries the
  command that checks it for the target it is for.

  **Corrected 2026-10-06 (S14 pass):** the rejected arrangement is the one in
  the tree. Since `74b55236` (2026-08-24) `ports/graphshell/web/Cargo.toml`
  carries its own `[workspace]` (line 16), the root lists it under `exclude`
  (root `Cargo.toml`, lines 156-167, whose comment says the `[workspace]`
  table is what actually keeps it out), and it restates the root's patches.

  **A third instance of the crypto-row hole surfaced in the same pass**, in
  `src/bin/h6_transfer_peer/source.rs`. It had stayed hidden because bin
  targets are only compiled by `--all-targets` or a workspace check — the
  gate that could not run. It carries the byte-iteration idiom inline: a
  bin links the library as an external crate, so `transfer::hex_digest`
  (private) is out of reach.

  **Receipts**: `cargo check --workspace --all-targets` on
  x86_64-pc-windows-msvc exits 0 with zero error lines, and `cargo check -p
  graphshell-web --target wasm32-unknown-unknown` exits 0. Use
  `--all-targets`: the lib-only form is what let a bin carry a compile
  error for two days.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_50_s14_phase_b12.md: the status records the
  2026-08-12 landing (`8595cd38`) with the census row as the one open item,
  and E-R4's fetchers (`254b23b6`) and graphshell-web's own workspace
  (`74b55236`) carry corrections.

## Done conditions

- ~~`cargo test -p mere-eidetic --features vector-index`~~ **`cargo test -p
  esp --features persistence`** passes with the moved round-trip tests
  (6 green, 2026-08-12).
- Canvas compiles with the bridge modules and their existing tests.
- `mere-embed` is gone from the tree and the workspace manifest; grep
  finds no `embed::` path outside the census/history record.
- The census §2 rows for `mere-embed` and the search plan's W4 wording
  point at the new homes.
