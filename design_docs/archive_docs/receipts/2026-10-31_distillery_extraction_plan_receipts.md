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

## 2026-11-03: consumer cutover

| Consumer | Pushed source cohort | Receiving checks |
| --- | --- | --- |
| Mere | `f673d313d09e6c7a043779e8c172a53f76e328a7` donor cut; this closeout changes manifests, locks and docs | workspace/all targets; optional inference/training/Chronicle composition; seven composition tests; five standalone roots |
| Eidetic | `1014cf48e1882ff926b55f3509ac3dbf5a5aec8e` | workspace/all targets |
| Knot | `a84ae72840e07b390eb5a14446f1badcc4d2754a` | root, knot-document and knot-site/all targets |
| Cleromancy | `3b65ebc01ec8475aa724f894adb60c93718c34bc` | workspace/all targets |
| Isocosm | `33ea4199568d4f858eb6051b53cd47b112bfd4df` | nine independent roots/all features/all targets |
| Mer3ly | `4ff6a1571c625088cf32a95212f60199bd734484` | three native roots; both wasm32 clients; 40 site tests |
| Turnstone | `4321a55482bcbe7d9bbc48e32c6a0c91e2dc82c5` | workspace/all targets, including Windows sandbox helper; two recycle tests |

All checks use their own locked graphs. The 24 independent consumer roots
have compact selected-source audits in `consumer-source-audits.jsonl`.
The final audit compares manifest and lock hashes with each working root.
Eidetic retains its already-qualified `genet-text` source `01d3f528`; other
Genet-bearing receiving roots retain `ad362723`. An exploratory metadata
normalization of Eidetic's lock was preserved separately and restored,
keeping its source intake bounded.

The additional Isocosm roots are cosmograph-host, shared/cosmograph,
shared/isomere, shared/isometer, shared/wing-integration, mesocosm, eponym and
mesocosm/crates/probes/parry-ground, alongside the main root. Mer3ly's roots
are the site, crates/repo-graph and crates/radio-mirror. Mere's standalone roots
are crates/cambium/examples/genet_web_smoke, crates/probes/murm-direct-phy,
ports/graphshell/web, ports/muniment-opfs-probe and support/luggage-comparison.
Ignored library/probe locks are preserved beside the gates, with their hashes.
These compilation receipts do not extend an earlier browser, headed, model
or physical-radio acceptance claim.

Turnstone's old Djinn door imports now use Ostiary and castellan-client
directly; four test fixtures take wallet sealers from Sporran. Its recycle
actor uses `pandect::furnace`, retaining application retention policy and
the held Athanor workshop dependency. No Pandect re-export shim was added.
Knot's standalone site requirement now accepts the current Servitor 0.0.3.
Registry producer-contract patches select each consumer's own Mere source,
preventing registry/Git copies of the same host contract.

The standalone Graphshell lock had selected incompatible digest generations
for Spareval's broad hash-library requirements. Its MD5 and SHA-1 selections
now match its SHA-2 digest generation, and its locked check passes. The
Cambium smoke override names the actual Genet Taffy 0.16 package at the
qualified revision. The direct-PHY probe's lock selects the p2panda fork
already declared in its manifest and forwards older Eidetic Git sources into
the receiving cohort. The fork tag itself is unchanged.

The final active-manifest scan covers all seven consumer repositories plus
the receiving Distillery main worktree; it finds no `merely-made/esp.git`.
Frozen historical receipts are excluded. All consumer full-tree hooks pass.
Mer3ly's ordinary main push starts its separate Pages pipeline; local native
and wasm compilation does not certify that deployment.

The extraction removes 367 tracked donor files including the lexical probe.
It completes the compute source-home cut and Pandect caller follow-through.
Athanor/Alembic workshop relocation retains D16's hold. The family promotion
plan separately records the next p2panda ownership seam: moving Stickleback's
network joining/sync mechanics through Murm while preserving Eidetic's one
validated store and Gemot's group policy. No full-module completion is claimed.
