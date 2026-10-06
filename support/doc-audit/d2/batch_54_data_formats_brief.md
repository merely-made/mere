# Batch 54: the data formats brief (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| 2026-10-06_data_formats_brief.md | current | yes | 12 | 11 | 0 | 1 |
| **Totals** |  |  | **12** | **11** | **0** | **1** |

**Totals: 1 doc, 12 claims checked (11 holds, 0 stale, 1 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere at the commit that adds the brief (2026-10-06), with the
other repositories at their checked-out heads that day. The coordinator who
wrote the brief judged it on the same day; an independent re-judgment can
supersede this record under ruling S34.

## 2026-10-06_data_formats_brief.md

- disposition: current
- status line: "**Status, 2026-10-06:** ruled (F1 to F5). The rule is in force for new files. Migrating the outliers (§4) and the postcard version-header audit (F4) are separate lanes." — accurate: yes
- claims checked: 12 — holds: 11, stale: 0, unverifiable: 1

### Stale claims

None.

### Contradictions

None.

### Recommended action

Re-judge when the outlier migration or F4's audit lands, since both change
§4.

### Notes

Each claim below was checked against the source in brackets, and holds:
- the per-repo data-file counts [`git ls-files` in each repository, with the
  filters the brief states];
- the repos declaring `serde_json` and `toml` [each repository's
  `Cargo.toml` files];
- Mere's `.wasm.toml` mod manifest
  [`crates/system/registry/src/mod_loader/loader/free_fns.rs:65` and `:110`];
- the wing's JSON pack manifests and Mesocosm's five process records
  [`git ls-files` in isometry];
- Livery's datasheet shape: `schema`, owner, `status`, `[sources.*]`
  [`genet/components/livery/properties.toml`];
- the postcard call-site counts per crate and repo [`git grep` for
  `postcard::to_`/`from_`, tests excluded];
- `pandect`'s CBOR use [`crates/system/pandect/src/wallet_grant/mod.rs:47`];
- hocket's `.hock` format and its reason [`hocket/README.md:22`,
  `hocket/design_docs/2026-05-18_initial_plan.md:1406`];
- `graph-kernel`'s codec-race crates as dev-dependencies
  [`crates/graph/graph-kernel/Cargo.toml`, `[dev-dependencies]`];
- `wing-formats`' version-header description
  [`isometry/shared/wing-formats/src/lib.rs`];
- the about-70 p2panda CBOR decode sites [the device pairing plan's CBOR
  audit, which records 73 calls in Mere and Knot];
- Mark's rulings F1 to F5 [this session's questions and answers,
  2026-10-06].

Unverifiable: the version-header grep in §3 is a name search, not a reading
of each record's encoding, as the brief says. F4's audit is the check.
