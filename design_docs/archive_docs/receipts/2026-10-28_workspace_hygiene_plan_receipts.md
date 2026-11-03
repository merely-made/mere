# Workspace hygiene receipts

## Registry releases, 2026-11-03

Owner: the registry release pass in
`../../mere_docs/implementation_strategy/2026-10-28_workspace_hygiene_plan.md`.

Compact gate records live at
`Code/testing/mere/registry-releases-20261103/gates.json`, with a 100 MB cap for
the entire pass. Failed gates are retained as records beside their repairs.
Build products use the stable `C:/t/cargo-targets/<repo>` paths. This pass
uses no isolated Cargo home. ESP's temporary collision worktree is recorded
below; it and its branch were removed after integration and push.

- Storage preparation: Eidetic `3a24b32`, `01dc67a`, `5e2bb76`, `9fe08ee`.
  The fork manifests preserve upstream licenses and library names. The Fjall
  fixture uses a Windows-supported path while preserving exact allocation and
  refusal assertions. Eidetic's unused network test dependencies were removed;
  its all-feature/all-target suite still passes 153 library and 3 integration
  tests.
- Mere release preparation: `d49dd8cf6`, including the unrelated pre-existing
  receipt-link commit through a normal merge. Its pre-commit line ceiling,
  Responsa, all-target Clippy and dependency-ban checks passed.
- Published and registry-visible: `mere-lsm-tree` 2.10.4, `mere-fjall` 2.11.2,
  `muniment` 0.2.1, `chartulary` 0.3.0 and `numen` 0.2.1.
- Registry-only packaged tests: Muniment 161; Chartulary 94; Numen 102 passed,
  1 ignored; Eidetic 153 library plus 3 integration tests. The initial Muniment
  all-feature package failure exposed missing fork APIs and was repaired by
  publishing the carried forks. The Fjall fork's own 32 unit tests pass after
  its Windows fixture repair; the LSM-tree bounded test filter passed 57 tests.

This is native automated and package-build evidence. It does not establish
headed, physical-device or other-platform runtime acceptance.

- Registry custody cohort: Dramatis `57a6760` published Gaz 0.0.3, Dramatis
  0.0.5, Castellan Client 0.0.2, Sporran 0.0.1, Gazette 0.0.2 and Castellan
  0.0.4. Every packaged all-feature/all-target gate passed before upload; the
  six package logs are retained with `gates.json`. The workspace passed 813
  libtest cases plus both custom residue harnesses. The client boundary
  fixtures passed and rejected the forbidden import with E0432.
- ESP `9e8a32d`: dependency-only storage alignment; 94 workspace tests pass
  with 1 ignored, and 88 actor tests pass. Its primary checkout remained
  unchanged on `distillery-receive` at `80b6ed8`. The collision worktree was
  fast-forwarded into main, pushed and removed with its branch.
- Mere Kernel's registry-only all-feature/all-target gate passed 530 tests
  with 2 ignored. Tabard's packaged suite passed 96 tests after removing an
  unused image-decoding dev feature request absent from published Genet Livery.
- Knot `272d819` widens only dependency bounds and keeps the existing source
  lock; its locked all-target check and commit Clippy hook pass. This prepares
  the Mere/Knot dependency cycle for receiving the new cohort.

## Completed release and receiving record, 2026-11-03

All sixteen archives are registry-visible. `registry-provenance.json` records
SHA-256, archive size, clean `.cargo_vcs_info.json` commit and registry creation
time for every version. The downloaded archives total 1,946,380 bytes; the
archives themselves were not retained.

| Owner | Published versions |
| --- | --- |
| Eidetic | mere-lsm-tree 2.10.4, mere-fjall 2.11.2, muniment 0.2.1, chartulary 0.3.0, eidetic 0.0.3, servitor 0.0.3 |
| Dramatis | gaz 0.0.3, dramatis 0.0.5, castellan-client 0.0.2, sporran 0.0.1, gazette 0.0.2, castellan 0.0.4 |
| Mere | numen 0.2.1, tabard 0.0.2, mere-kernel 0.0.3, pandect 0.2.0 |

Pandect's registry-only packaged suite passes 338 tests. Its wallet surface
belongs to Sporran, and the published 0.2.0 package describes that boundary.
The aligned Servitor suite passes 98 tests. All sixteen packaged dependency
closures were qualified without workspace Git or path overrides.

| Repository | Pushed main commit | Qualification |
| --- | --- | --- |
| Eidetic | `9fe08ee2c793064fc65a2275c7e3ebe7eb9997c5` | Registry storage release closure; counts above |
| Dramatis | `1284558873ae8610aae6e7e1e4abe0619f02af71` | Workspace 813 libtest cases and 2 residue harnesses; package and client boundary checks |
| ESP | `9e8a32d43cca0160f7c7924db3764212fb1d9d61` | Workspace 94 pass, 1 ignored; actor 88 pass |
| Mere | `47dce59eef5d38127d30dee0e9400a4dc645375b` | Locked workspace, all targets; commit hooks |
| Knot | `e1ec7296cd11665be05100e875c0e0813552738d` | Locked all targets, 83.73 seconds; commit hooks |
| Cleromancy | `7d2e1fd797524ebf374d20049ea1b938bc32f69c` | Locked all targets, 191.77 seconds; commit hooks |
| Isocosm | `af9e0e96c50929054f4a20b9ca0a3e01455da899` | Nine workspaces, locked all features and targets; full-tree and commit hooks |

Receiving consumers select Mere `47dce59ee`, Eidetic `9fe08ee`, ESP `9e8a32d`
and Genet `ad362723109b` where applicable. Mere's transitional Knot lock
`272d819` breaks the receiving cycle; the final Knot commit tightens the
requirements to the qualified releases. Pin-only and documentation commits do
not trigger reciprocal source updates. Subsequent Mere changes close this
record without changing the qualified source commit.

`receiving.jsonl` retains individual runs, including the failed Isometer
feature gate and its repair. `isocosm-final-sources.json` records the last
passing run for each of the nine manifests. All nine final lock hashes match
those receipts; every locked Mere, Eidetic and Genet package uses the selected
commit. Five formerly ignored standalone locks are now tracked.

Isometer's resident-ground feature selects `conatus/burn` and Burn 0.22.1.
`resident-ground.json` and `resident-ground-basis.json` record the headless
RTX 4060 Laptop GPU / Vulkan gate: shared raw/Burn allocation, zero commit
allocator growth, epoch refusal and exact replay pass; 184 pixels change.
Current device-wide allocation totals are 394,640 bytes / 17 allocations, with
a 393,216-byte initial atlas. These are recorded separately from the older
source receipt in Mesocosm's engine ecology review. `parry-ground.log` also
records a passing CPU projection, delta, query, stale-revision refusal and
bit-identical replay probe. No simulation or stored-data format changed here.

The working toolchain is Rust 1.99.1. These gates do not claim new headed,
other-platform runtime or shared-MSRV acceptance. No isolated Cargo home,
target or worktree remains; stable per-repository targets were reused. The
ESP primary checkout remains on its unrelated `distillery-receive` work.
Failed Muniment package inputs were preserved compactly in
`failed-muniment-package-basis.json` before removing generated package copies.

Retained compact receipts are under 1 MB of the 100 MB lane cap.
`final-repositories.json` records the final repository and remote-main audit.
