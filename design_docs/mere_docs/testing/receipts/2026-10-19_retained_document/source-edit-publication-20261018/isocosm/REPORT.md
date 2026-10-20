> Copied 2026-10-19 from the M4-local `~/Code/testing/source-edit-publication-20261018/isocosm/REPORT.md`; see the [receipts index](../../README.md). Absolute `/Users/markik/...` paths below are host-local reproduction paths, not links.

# Isocosm receiving qualification: published source-edit supplier

2026-10-18. All receiving checks passed on Isocosm `183a2f67` with its pending
adapter edits and the published Mere supplier
[`70deef2c8cf3d3410ae35b496a9396514070b24f`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f).
The consumer remains unstaged, uncommitted and unpushed for parent review.
Earlier testing receipts were not overwritten.

## Resolution and lock scope

No explicit config patch or environment override was used. The canonical
manifest remains optional Git `branch = "main"`, version 0.1.0. Both scoped
`cargo update -p mere-source-edit` and the version-qualified form could not
select the previously path-patched package once its override was removed.
An ordinary unpatched `cargo check -p isometry-system --features pack-authoring
--bin pack-author` generated the Git lock entry. Attempts and results are in
`cargo-update*.log` and `resolve.log`.

The lock source is exactly
`git+https://github.com/merely-made/mere.git?branch=main#70deef2c8cf3d3410ae35b496a9396514070b24f`.
All existing package blocks are retained exactly except `isometry-system`'s new
dependency edge. No existing supplier revisions changed and no broader lock
coupling or URL spelling workaround was required.

`metadata.json` identifies the package as that Git source and locates its manifest
at `/Users/markik/.cargo/git/checkouts/mere-9d8e871c24d9ba76/70deef2/crates/system/source-edit/Cargo.toml`.
The checkout's actual Git HEAD was independently checked against the supplier
commit. `qualification.json` records this metadata and hashes of the resolved
Git supplier files, rather than a sibling working-tree package. The default
normal tree in `default-tree.log` excludes the optional kernel, Taplo and Rowan.

## Actual receiving checks

- 75 feature-enabled tests: 49 library, 15 authoring, 2 campaign assertions,
  2 native packs, 2 subprocess authoring tests and 5 watchtower tests.
- 58 default tests; all-feature/all-target package check, scoped all-target
  Clippy, Rust 1.97.1 binary check, build, formatting and diff checks passed.
  Existing warnings remain; no package-wide strict warning claim.
- The rebuilt Git-backed binary passed 59 commands across five fresh CLI
  processes using a scratch real pack: reviewed insertion, explicit apply/save,
  stale/duplicate/partial-index refusals, preserved comments/unknown content,
  invalid-draft save/reopen/repair, external-conflict recovery and real loader/
  fixture execution. Original pack and unchanged scratch assets were hash-checked.
- All 64 ready/command records match the prior deduplicated adapter workflow
  after replacing only absolute scratch-pack directory paths.

`checks.json` contains exact commands, statuses and adjacent raw logs.
`workflow/` contains the transcript, sources and fresh-process/fixture results.
`protocol-comparison.json` records the historical comparison. `qualification.json`
contains derived counts, source and binary receipt hashes, exact changed paths,
lock scope and Git-only metadata evidence. The owning Isocosm plan/index now
link the published supplier and distinguish receiving from consumer publication.

## Reproduce without overrides

From `/Users/markik/Code/repos/isocosm`:

```sh
python3 /Users/markik/Code/testing/source-edit-publication-20261018/isocosm/run-checks.py
cargo metadata --locked --format-version 1 --features pack-authoring > /Users/markik/Code/testing/source-edit-publication-20261018/isocosm/metadata.json
python3 /Users/markik/Code/testing/source-edit-publication-20261018/isocosm/qualify.py /Users/markik/Code/testing/source-edit-publication-20261018/isocosm/reproduction
```

Use a new workflow output directory. No CLI protocol, parser/layout/schema,
explicit save, admission, runtime or original fixture changes were needed for
this receiving pass. No GUI, crash-recovery or consumer publication is claimed.
