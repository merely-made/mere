# Workspace hygiene receipts

## Registry releases, 2026-11-03

Owner: the registry release pass in
`../../mere_docs/implementation_strategy/2026-10-28_workspace_hygiene_plan.md`.

Compact gate records live at
`Code/testing/mere/registry-releases-20261103/gates.json`, with a 100 MB cap for
the entire pass. Failed gates are retained as records beside their repairs.
Build products use the stable `C:/t/cargo-targets/<repo>` paths. This pass
creates no isolated Cargo home or Git worktree.

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
