> Copied 2026-10-19 from the M4-local `~/Code/testing/retained-lowering-profile-20261016/REPORT.md`; see the [receipts index](../README.md). Links to these files, which stay local, are shown as plain paths: `src/profile.rs`, `vendor/taplo-rowan17/src/dom/mod.rs`, `vendor/taplo-rowan17/src/dom/node/nodes.rs`, `vendor/taplo-rowan17/src/util/shared.rs`.

# Wide-table lowering profile

**Date:** 2026-10-16. **Status:** completed isolated profiling; no optimization
or product adoption.

## Finding

DOM construction dominates the 4,096-key lowering path: **116.303 ms out of
124.426 ms**, about **92.8%** by the median per-sample share across three fresh
processes. Interleaved uninstrumented `Snapshot::lower` controls have a median
of **125.462 ms**, consistent with the instrumented pass. Stage boundaries,
local/result destruction, and controls are all retained in the raw results.

The cause is repeated copy-on-write cloning of accumulated table entries during
DOM construction. The counter build observes **8,386,657 cloned vector entry
pairs**, exactly **4,096 × 4,095 / 2 + 97**. The 97-pair addition is fixed pack
scaffolding and remains the same across all tested widths. `Entries::clone`
also clones the lookup map; the pair counter counts only the `all` vector's
entries, not both collections, allocation bytes, or deep subtree copies.

## Stage results

Median of per-process sample medians, release build, local Apple M4, three
fresh processes. Small fixtures have 12 repetitions/process; wide fixtures
have five except 4,096 keys, which has three. The second process reverses the
fixture order. Counts are disabled in every timing run.

| Stage on wide 4,096 | Median ms |
|---|---:|
| `dom_construct` | 116.3028 |
| `dom_validate` | 0.0863 |
| `dom_to_json` | 0.7029 |
| `independent_parse` | 3.0998 |
| `independent_json_compare_drop` | 0.6680 |
| `schema_input_clone` | 0.0802 |
| `schema_deserialize` | 0.0618 |
| `manifest_validate` | 0.0020 |
| `canonicalize` | 1.6950 |
| `canonical_encode` | 0.0689 |
| `prefix_sha256` | 0.1341 |
| `result_and_local_cleanup` | 1.5272 |
| `result_drop` | 0.0750 |

Individual stage medians need not sum exactly to the median wall time. Timing
map insertion/clock overhead is in wall totals but outside most stages. The
interleaved original lowering function is the end-to-end control; it is not
rewritten to use the timers. Opening/parsing the retained snapshot is outside
both lowering measurements. Every timed pass constructs a fresh DOM, rather
than measuring a cached result. Additional warm-DOM validation/JSON controls
are in the summary.

The independent parser and canonicalization are measurable but small beside
DOM construction on this fixture. Typed manifest validation and SHA-256 are
also small here; this is not a basis for removing checks or changing identity.

## Width sweep and source mechanism

| Fixture | DOM construction ms | Lowering wall ms | DOM share |
|---|---:|---:|---:|
| real | 0.022 | 0.082 | 27.6% |
| annotated | 0.027 | 0.100 | 26.8% |
| wide_256 | 0.439 | 1.005 | 44.3% |
| wide_512 | 1.642 | 2.770 | 58.3% |
| wide_1024 | 7.016 | 9.042 | 75.5% |
| wide_2048 | 28.731 | 33.528 | 85.9% |
| wide_4096 | 116.303 | 124.426 | 92.8% |

The wide fixtures append a flat table under `[vendor.wide]` to the annotated
pack. The 4,096-key fixture is the same 64,446-byte source as the document proof.
Wide-fixture source generation is outside all timers. No scripts or assets run.

Taplo's source provides the causal chain:

1. `Table::add_entry` (`vendor/taplo-rowan17/src/dom/node/nodes.rs`, local only) calls
   `self.inner.entries.update(...)` for each insertion.
2. `Shared::update` (`vendor/taplo-rowan17/src/util/shared.rs`, local only) calls
   `load_full()`, then `Arc::make_mut`, then stores the replacement. The
   ArcSwap slot still retains the old Arc while `make_mut` runs, so the loaded
   allocation is shared even without another application reader.
3. `Entries` (`vendor/taplo-rowan17/src/dom/mod.rs`, local only) contains a lookup hash map
   and an ordered vector. Cloning it copies the accumulated containers before
   adding the next entry.

The separate counter build replaces derived `Clone` with equivalent field
cloning plus atomic counters. The feature-off timing build keeps the derived
implementation. These counters show the triangular copy count directly:

| Wide keys | Clone calls during construction | Cloned vector pairs | Largest cloned width |
|---:|---:|---:|---:|
| 256 | 307 | 32,737 | 255 |
| 512 | 563 | 130,913 | 511 |
| 1,024 | 1,075 | 523,873 | 1,023 |
| 2,048 | 2,099 | 2,096,225 | 2,047 |
| 4,096 | 4,147 | 8,386,657 | 4,095 |

DOM validation and JSON conversion add zero further `Entries` clones in this
valid corpus. The combination of source inspection, exact triangular counts,
and width scaling supports quadratic construction work. It does not establish
all-input parser complexity or explain every cost in invalid/deep documents.

## Correctness and scope

Both the normal and counter-enabled configurations pass **11 release tests**:
the four copied manifest validator tests, six retained-resource integration
tests, and one new timed-path equivalence test. Formatting and Clippy with
warnings denied pass for the probe; the vendored library retains its existing
unused-method warning. The size sweep also checks complete canonical values,
typed manifests, and domain hashes against the unchanged lowering function for
every fixture. No validation, canonicalization, hashing, or admission algorithm
was optimized or bypassed.

The original proof's parser/validator source hashes were checked after this
run and are unchanged. Product repository status and HEAD observations are in
[isolation.json](results/isolation.json); product operations were read-only.
The new code, optional counters, and receipts stay in this testing directory.
No shared documentation update, commit, push, or product edit was performed.

## Implication

The next bounded optimization experiment should target private DOM table
construction: batch entries while the builder has exclusive mutable ownership,
then publish the shared representation. Preserve duplicate-key/table conflict
checks, source associations, and order. Keep the current lowering path as an
independent correctness control and repeat the width sweep before claiming a
benefit. Parser caching, resource-history sharing, and faster hashing address
other concerns and are not needed to explain this measured bottleneck.

No optimization was implemented in this pass. The original proof's syntax,
text-only generator ABI, save-coordination, crash-recovery, and product-adoption
limits remain in force. The timings differ from the earlier document proof's
composite measurements on a shared host; the profiling change is not a speedup
claim. Raw process samples record variation; the exact clone counts give the
more stable algorithmic evidence.

## Reproduction and artifacts

Run `python3 run.py` in this directory. It uses the retained Cargo.lock, tests
both configurations, runs three fresh timing processes, then builds and runs
the counter version separately. No product checkout is needed to reproduce.

- [summary.json](results/summary.json): stage medians, controls, width sweep,
  and clone counts.
- `results/profile-1.json` through `profile-3.json`: every measured stage sample.
- [clone-counts.json](results/clone-counts.json): counter-only results.
- [instrumentation.patch](results/instrumentation.patch): changes to copied
  source/manifest; the added stage harness is profile.rs (`src/profile.rs`, local only).
- [tests.log](results/tests.log) and [counter-tests.log](results/counter-tests.log).
- [environment.json](results/environment.json) and
  [dependency-tree.txt](results/dependency-tree.txt).
- [baseline source hashes](baseline/source-hashes.json),
  [original proof report](../retained-document-20261016/REPORT.md), and
  [syntax/save policy](../retained-document-20261016/SYNTAX.md).
