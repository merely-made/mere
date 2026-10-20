> Copied 2026-10-19 from the M4-local `~/Code/testing/shared-source-edit-20261018/isocosm/REPORT.md`; see the [receipts index](../../README.md). Absolute `/Users/markik/...` paths below are host-local reproduction paths, not links.

# Isocosm shared source-edit adapter qualification

2026-10-18. Passed against fetched/fast-forwarded Isocosm `183a2f67`, with
uncommitted `mere-source-edit` 0.1.0 supplied by one explicit local Cargo patch.
Nothing was committed or pushed. Historical qualification artifacts are unchanged.

`Session::edit` now uses `Proposal::prepare` and `application`; preset insertion
gets preview and edits from the same validated plan. Host document/open-instance/
revision stamps stay internal. The legacy CLI JSON and explicit edit/save flow
remain unchanged. Isocosm retains bounded preflight, history publication, TOML
batching/indexing, real schema and expected-manifest checks, unknown-header/layout
refusals and all save/conflict/admission policies. Undo remains app-owned.

## Actual results

- 74 feature-enabled tests: 49 library, 14 authoring, 2 campaign assertions,
  2 native packs, 2 subprocess CLI and 5 watchtower tests.
- 58 default tests; all-target/all-feature package check, scoped all-target
  Clippy, Rust 1.97.1 binary check, rustfmt and diff checks passed. Existing
  package/dependency warnings remain; no package-wide `-D warnings` claim.
- Real scratch-pack workflow: 59 commands in five processes passed, including
  reviewed preset insertion, explicit application/save, stale/duplicate/partial
  index refusals, comments/foreign-content preservation, invalid-draft reopening
  and repair, external-conflict recovery and actual loader/fixture execution.
  Original pack and unchanged scratch assets were hash-checked.
- All 64 ready/command records match the earlier private CLI workflow after
  replacing only absolute scratch-pack directory paths. See
  `protocol-comparison.json` for transcript paths and normalization.
- Default normal dependency tree contains no `mere-source-edit`, Taplo or Rowan.
  Lock diff adds only the new local package and its optional dependency edge;
  all existing package blocks except that dependency edge are retained exactly.

## Reproduction and records

From `/Users/markik/Code/repos/isocosm`, Cargo checks explicitly use:

```sh
cargo --config /Users/markik/Code/testing/shared-source-edit-20261018/isocosm/local-source-edit.toml test --locked -p isometry-system --features pack-authoring
```

The exact config is retained as `local-source-edit.toml`:

```toml
[patch."https://github.com/merely-made/mere.git"]
mere-source-edit = { path = "/Users/markik/Code/repos/mere/crates/system/source-edit" }
```

No other Mere package is locally patched. For the external Clippy subcommand,
pass the same `--config` after `cargo clippy`; a global pre-subcommand config
was not forwarded into its check process. `run-checks.py` records exact commands
and process status in `checks.json` and adjacent logs. `metadata.json` records
the binary target directory. After building, repeat the workflow in a new directory:

```sh
python3 /Users/markik/Code/testing/shared-source-edit-20261018/isocosm/qualify.py /Users/markik/Code/testing/shared-source-edit-20261018/isocosm/reproduction
```

`qualification.json` records changed paths, parsed test counts, source/supplier
hashes, patch contents and dependency/lock checks. `workflow/` retains the exact
transcript, final/invalid/conflict sources and actual process results. The
workflow receipt hashes correspond to the supplier at workflow execution;
the final root receipt captures the supplier after its later documentation-only
comment clarification. No API/method behavior changed in that clarification.

## Publication boundary

The manifest declares optional Git `mere-source-edit` from canonical
`https://github.com/merely-made/mere.git`, `branch = "main"`, version 0.1.0.
Actual unpatched `cargo tree --locked -p isometry-system --features pack-authoring`
failed (exit 101): the package is not present on fetched Mere main. See
`remote-resolution.log`. The lock records the path-patched package, not a portable
remote lock. After supplier publication regenerate only that package's remote
lock entry and qualify unpatched builds. This qualifies assembled local source,
not publication, a GUI, crash recovery, durable cross-process proposals or runtime
admission. The private protocol still carries session `base` plus edits, not a
serialized shared proposal token.
