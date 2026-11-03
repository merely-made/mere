# Distillery extraction receipts

## 2026-11-03: receiving and donor cut

- Main receives the prior filtered history at Distillery `63460e0660ebd02ddf43df87dd208f334cceeb7e`,
  with parents ESP main `9e8a32d43cca0160f7c7924db3764212fb1d9d61` and the existing
  receiving branch `80b6ed825f6e6c51ba6d94ba4793058c93fd7490`. The primary ESP
  checkout stays on that clean receiving branch; only its shared origin URL changes.
- GitHub repository renamed to `merely-made/distillery`. Runtime receiving pins
  `66108581b3d2ca66eb42a282eccc8ec5d4039e4e`; Eidetic's source intake is
  `1014cf48e1882ff926b55f3509ac3dbf5a5aec8e`. The later lexical-probe move is
  Distillery `cc9c7dac618f24ca6ef94f07b8b0f5496c70b9b6`, with unchanged runtime blobs.
- The initial donor cut removes 363 tracked files after receiver push and Git-blob
  comparison: 331 are identical; differences are manifests, generated locks,
  current README paths, relocated tests/probes and stricter source provenance.
  Runtime modules and frozen receipts match their donor Git blobs. The four-file
  lexical wasm probe follows separately from Mere `a9d19631d754f8b684bd92f39b08de034b35a7a8`;
  its original hashes and receiving revision are recorded before donor removal.
- Graphshell and the resident keep their own exact board-fixture snapshots.
  Producer tests have a private allocator fixture, copied byte-for-byte from
  the resident fixture. These moves remove test paths into another source home.
- Distillery's final runtime cohort passes 278 default tests, with one existing
  ignored benchmark; the actor library suite passes 88 during receiving. Locked
  all-feature/all-target compilation passes. Each of the six standalone probe
  workspaces passes its own locked compilation, including wasm32 browser builds.
- The relocated lexical module builds for wasm32 and runs under Node: canonical
  length 801, BLAKE3 `4d0f647fa7f15639aad9591ae9e466d48d9d029ef581d3a6d25ca10e5258aa01`.
  The mesh's native `lexical_determinism_receipt` passes against that same digest.
- Mere's whole workspace checks locked with all targets. Its resident also
  checks inference embedding/GPU/stages, autodiff/GPU training and the Chronicle
  receipt host together. Each of 21 audited contract/storage/compute package
  names has one source, with Genet kept at `ad362723109b7e1738dfce13a625cf1a1b5e9ed1`.
  The existing app-host guest-projection build warning remains; it also appears
  in the identity lane's pre-extraction baseline. No guest, headed model or
  physical network acceptance is added by these compile receipts.
- Twelve previously ignored wasm-bindgen bundle files (44,776,733 bytes) are
  preserved with SHA-256 hashes under the lane's `preserved-web-bundles/`, rather
  than becoming untracked files when their old per-directory ignores leave.
  They are historical outputs, not new acceptance receipts.
- Run records, source comparisons, fixture-copy hashes and exact commands are
  at `Code/testing/esp/distillery-extraction-20261103`, under the 100 MB receipt
  cap. Cargo uses the stable repository targets. Automatic approval review
  rejected removing the 849,687,075-byte marker-identified probe build cache
  from the collision worktree, reporting only "blocked by policy"; it remains
  owned by this extraction until cleanup is permitted. No isolated Cargo home
  was created. Consumer source/locked-check closeout follows below.

Mere's direct receiving gates also pass: the two Chronicle-route cases and
five Distillery-lane cases, including an actual mesh job and ordered shutdown.
The full-tree line ceiling, Responsa, Clippy and dependency-ban checks pass.
