> Copied 2026-10-19 from the M4-local `~/Code/testing/retained-dom-batch-20261016/REPORT.md`; see the [receipts index](../README.md).

# Bounded DOM construction batching

**Date:** 2026-10-16. **Status:** completed isolated optimized/control comparison.

## Result

On the same 4,096-key fixture, batching consecutive entries reduces DOM
construction from **356.069 ms** to
**2.240 ms**
(**159.0×** in these local runs). Interleaved whole-lowering
controls improve from **380.106 ms** to
**14.472 ms**, or **26.3×**.
These are paired local release observations, not portable performance guarantees.

The separate clone counters fall from **8,386,657** to
**41 cloned vector entry pairs** during DOM construction.
The wide contiguous table no longer incurs the triangular copy term. Both
implementations produce the same values, errors, order and measured source
associations in the bounded comparison corpus.

## Exact change

The optimized library differs from the unchanged profiling control in two
Rust files, plus package/library naming in its manifest for coexistence:

- `dom/node/nodes.rs`: extract the original `add_entry` closure body into
  `add_entry_into`. `add_entry` still uses the same helper for a single item;
  the private construction helper `add_entries` calls it repeatedly inside
  one `Shared::update`.
- `dom/from_syntax.rs`: batch consecutive root `ENTRY` syntax nodes, stopping
  at every header or other node; batch inline-table entries similarly.

The per-entry helper body is identical to the original checks apart from
indentation, verified by [source integrity](results/conflict-helper-integrity.json).
Duplicate checking, pseudo-table merging, error creation, additional source
association updates, lookup insertion and ordered-vector insertion all remain
in that body. Header and array-of-table handling remains unchanged. The
[patch](results/optimization.patch) is retained.

Batching happens inside synchronous `FromSyntax` construction before the DOM
is returned. The construction closure owns one mutable entry container for
that batch, rather than cloning it for each entry. It does not alter
`Shared::update`, concurrent read behavior, public DOM mutability, syntax trees,
resource revisions, admission, canonicalization, or hashing.

This is a bounded fix for contiguous entry runs and inline tables, not a new
general DOM builder. Dotted/pseudo-table merges still take their existing
recursive single-entry path; many alternating headers or separately revisited
tables can still clone populated containers. The root path uses a temporary
vector for its batch, an O(N) construction allocation. Peak memory/RSS and
all-input complexity are not measured in this experiment.

## Correctness comparison

Both libraries pass **101 inherited taplo tests each** (parser, invalid-input,
formatter and DOM rewrite cases). Both resource backends pass **12 release
tests each**: four inherited pack-validator unit tests, six lifecycle
integration tests, the stage-path equivalence test, and a direct differential
test. The optimized counter-enabled configuration passes the same 12 tests.
Formatting and Clippy with warnings denied pass for the probe; the vendored
libraries retain the same pre-existing unused-method warning.

The [differential receipt](results/comparison.json) records 20 named cases,
3,075 UTF-8-safe prefixes, 360 deterministic edits, and the same five wide-table
sizes. Named cases cover the real pack, annotated pack, Unicode/CRLF, edits,
duplicate root/inline keys, dotted-key pseudo merges and duplicates, repeated
or scalar/table headers, inline-table extension conflicts, array-of-table
conflicts and ownership, comments and ordering, and incomplete input.

For syntax-error input, the comparison checks lossless tree text, kinds,
ranges, token text, and syntax diagnostics. For syntax-valid input (including
semantic conflicts) it additionally checks semantic error messages, ordered
DOM traversal paths, each key's primary/additional source ranges, node and
descendant source ranges, full values when valid, and formatter output. Values
alone would not establish conflict behavior or ordering, so these observations
are compared directly. This is bounded equivalence, not a formal proof or an
exhaustive malformed-input/fuzz qualification.

Inherited lifecycle tests continue to exercise immutable snapshots, fenced
handles/edits, undo/admission separation, typed manifest validation, save/reopen,
conflict/failure behavior, worker snapshots and bounded history. The original
proof's restricted syntax and text-only generator ABI remain in force. The
profile harness checks typed manifests, canonical values and hashes against
the selected lowering path, and the paired runner requires matching domain
hashes across both implementations for every width.

## Same width sweep

Three fresh processes per backend; execution order and fixture order are
reversed in the middle pair. Timings are medians of process sample medians.
Stage counters are disabled in these binaries. Whole lowering uses the same
interleaved uninstrumented control as the previous profile, including result
destruction. Retained source parsing/opening is outside lowering timings.

| Fixture | Baseline DOM ms | Batched DOM ms | Baseline whole lower ms | Batched whole lower ms |
|---|---:|---:|---:|---:|
| real | 0.0222 | 0.0177 | 0.0800 | 0.0771 |
| annotated | 0.0250 | 0.0207 | 0.0939 | 0.0912 |
| wide_256 | 0.6496 | 0.1815 | 1.5573 | 1.0320 |
| wide_512 | 2.6981 | 0.3229 | 3.9408 | 2.0428 |
| wide_1024 | 12.1240 | 0.4402 | 16.9550 | 3.7082 |
| wide_2048 | 68.1954 | 0.8225 | 70.9669 | 8.0353 |
| wide_4096 | 356.0686 | 2.2401 | 380.1060 | 14.4725 |

The real demo is 850 bytes, the annotated variant is 1,115 bytes, and the
4,096-key `[vendor.wide]` fixture is the same 64,446 bytes as the preceding
proof/profile. These small-case measurements are noisy; raw samples and
process medians are retained rather than implying statistical significance
for tiny changes. The large width result has independent algorithmic evidence
from clone counters.

| Wide keys | Baseline cloned vector pairs | Batched cloned vector pairs |
|---:|---:|---:|
| 256 | 32,737 | 41 |
| 512 | 130,913 | 41 |
| 1,024 | 523,873 | 41 |
| 2,048 | 2,096,225 | 41 |
| 4,096 | 8,386,657 | 41 |

The counter feature replaces derived `Entries::clone` with equivalent field
cloning and atomics in both libraries. It counts the ordered vector's entries;
each clone also clones the lookup map. These are work counts, not allocation
bytes or deep tree-copy counts. Counter binaries are separate from timing
binaries. The optimized path still performs unchanged independent parsing,
validation, conversion, canonicalization and hashing, so those stages can now
form a larger fraction of the remaining total.

## Scope and reproduction

Run `python3 run.py` here. It checks the per-entry helper's unchanged body,
builds both backends with the retained Cargo.lock, runs inherited/resource
suites and differential checks, executes three paired width sweeps, and runs
separate clone-counter builds. `python3 write_report.py` regenerates this
report from the retained summary. Dependencies may be fetched on first use;
no product checkout is needed to rerun the experiment.

[isolation.json](results/isolation.json) records original profiling source
integrity and product HEAD/status observations. All implementation changes,
code, fixtures and results are in this new testing directory; previous probes
and product code are unchanged by this work. No product/documentation adoption,
commit, or push was performed.

The experiment supports keeping consecutive-entry batching as a tested
candidate for this isolated backend. It does not certify global linearity,
peak memory, other consumers/platforms, crash recovery, a full TOML grammar,
or production compatibility. Those boundaries remain separate from the
measured improvement.

Artifacts: [summary](results/summary.json), [comparison](results/comparison.json),
[optimization patch](results/optimization.patch), [environment](results/environment.json),
[dependency tree](results/dependency-tree.txt), and per-backend test, timing and
counter files under `results/`.
