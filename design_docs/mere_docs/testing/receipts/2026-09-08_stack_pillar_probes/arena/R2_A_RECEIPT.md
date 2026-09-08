# R2-A: pinned Rust arena experiment, 2026-09-08

This is a direct test of Genet's Rust `ScriptedDom` and `Pins`, not a model of
the arena. It does not instantiate Boa, Nova, a retained document session,
Ortet, a renderer or a server. No production source was edited.

## Source and configuration

- Genet revision: `ee0b314b3e9ac4a2fadb07fb7816990fa3f2b71d`.
- Source copied from Git, excluding concurrent parser/runtime checkout edits.
  `source-manifest.json` records the original component and patch archives and
  per-file SHA-256 digests. The external harness carries Genet's path patches.
- Rust `1.97.1 (8bab26f4f 2026-07-14)`, native Windows x86_64, default library
  features, Cargo dev test profile unless stated otherwise.
- Separate Cargo target and private Cargo home. The shared cache was locked;
  registry metadata and crate archives were copied into the private home.
  Resolution and builds used `--offline`; the final `probe/Cargo.lock` is
  preserved. No shared Cargo lock file or configuration was modified.
- `debug-run.json` and the variant run records contain exact commands, exit
  codes and manifest, lock, harness, source-manifest and runner digests.

## Baseline result

`python run_probe.py debug` ran seven assertions against unmodified source.
Cargo returned 101: **5 passed, 2 failed**. Build and run took 551.69 seconds
on a machine with concurrent builds; this is not a performance benchmark.

| Named assertion | Native 64-bit debug result |
|---|---|
| `control_remove_child_retains_pin_until_release` | Pass: ordinary detachment retains a pinned node; unpinning permits collection |
| `text_replacement_retains_pin_observers_off` | **Fail: child already absent before collection and still absent afterward** |
| `text_replacement_retains_pin_observers_on` | Pass |
| `fragment_replacement_retains_pin_observers_off` | **Fail: child already absent before collection and still absent afterward** |
| `fragment_replacement_retains_pin_observers_on` | Pass |
| `foreign_handle_refuses_same_local_slot` | Pass with the debug arena fence enabled |
| `control_released_ids_do_not_alias_after_churn` | Pass across 1,000 detach/collect cycles; live count returns to baseline |

This reproduces the source-review hypothesis: the two replacement operations
invoke immediate subtree deletion with observation disabled. Later pin-aware
collection cannot preserve a node that was already removed. The observer
toggle and ordinary-detachment cases are controls against blaming collection
alone. The harness fails on the desired retention contract; it does not
relabel those failures as conformance passes.

## Fence-disabled configuration result

`unchecked-Cargo.toml` adds only this package-level configuration:

```toml
[profile.dev.package.genet-scripted-dom]
debug-assertions = false
```

`python run_probe.py unchecked` returned 101: **4 passed, 3 failed** in
75.20 seconds including incremental compilation. Both retention failures
remain, and `foreign_handle_refuses_same_local_slot` now fails with
`foreign=NodeId(2), local=NodeId(2)`. Disabling the debug fence makes a foreign
handle resolve to a live local slot.

This directly exercises the fence-disabled Rust configuration. Dependencies
remain in the dev profile; this is **not** a full optimized-release build or a
wasm run. It neither measures boundary-handle overhead nor selects a handle
representation. `unchecked.log` and `unchecked-run.json` preserve the result.

## Disposable retention correction

With the baseline Cargo configuration restored, `prepare_candidate.py`
replaces the two `release_subtree(child)` calls in text/fragment replacement
with orphan-until-collection behavior. It changes only the scratch snapshot.
`retention-candidate.patch` preserves the exact semantic delta;
`candidate-source.json` records baseline/candidate source and patch hashes.
The source manifest continues to describe the baseline, so the candidate
receipt requires **both that baseline and the recorded overlay**.

`python run_probe.py retention-candidate` returns 0: **7 passed, 0 failed**
in 43.73 seconds including incremental compilation. The same harness, lock and
baseline Cargo configuration were used. This is evidence for the narrow
retention correction, not a production fix or completion of G5. The native
debug fence is enabled for this candidate run; the foreign-handle defect is
not corrected by this patch. Replacement-specific long-lived memory costs and
the full JS root/mutation matrix remain unmeasured.

## Reproduction

Copy this artifact directory to a writable experiment location. Restore source
with `python setup_source.py <path-to-genet-repository>`, then run:

```text
cargo test --locked --manifest-path probe/Cargo.toml -- --nocapture
```

For the fence probe, replace `probe/Cargo.toml` with `unchecked-Cargo.toml`
and rerun. For the retention candidate, restore `baseline-Cargo.toml`, then
run `python prepare_candidate.py` and rerun Cargo. Source starts pristine in
each independent replay; do not apply the candidate twice.

The baseline is expected to fail the two named retention assertions. The saved
`run_probe.py` additionally uses a sibling private `cargo-home` and `target`;
that cache was a local execution workaround, not required by the harness.
The private cache is omitted from these artifacts. Exact offline runner replay
requires separately seeding it; manual locked replay can use a normal Cargo
cache and network access.
The source archive does not include WPT. An initial full-worktree attempt was
interrupted and removed in favor of the component snapshot.

## Limits and next gate

Injected Rust pins establish an arena defect; they do not prove backend
reflector rooting, range/observer queued roots, owning-document semantics,
adoption, or shadow-tree retention. Those remain G5 work. Hosted rendering and
worker wake remain O5 work. The two-site experimental correction
is a diagnostic patch and must not be treated as a complete G5 implementation.
