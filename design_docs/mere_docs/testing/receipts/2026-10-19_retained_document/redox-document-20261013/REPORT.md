> Copied 2026-10-19 from the M4-local `~/Code/testing/redox-document-20261013/REPORT.md`; see the [receipts index](../README.md). Links to these files, which stay local, are shown as plain paths: `results/edited-monsters.json`, `results/pre-capacity-fix-source.rs`, `src/source_document.rs`.

# Retained editable document probe — 2026-10-13

## Finding

A retained editable document is a useful application resource. Existing Rust
libraries already provide substantial parts of it: Sonic for a fast mutable
JSON arena, and `toml_edit` for source-aware TOML editing. A bounded source-backed
JSON overlay adds exact preservation of untouched bytes with a much smaller
retained index. These measurements support a document lifecycle API around
format-aware backends; they do not establish a need for a new universal token IR.

On the real 334-record SRD import, Sonic's parse plus three field queries is
6.75× faster than `serde_json::Value`. The overlay
is 3.95× faster and retains
18.27 KiB of index/cache/edits,
in addition to its shared 311.84 KiB source. A narrow typed name extractor has
similar first-read cost (166.30 µs),
so selective import alone does not justify a general document framework.

## Scope and inputs

- Original JSON: `isocosm@e5f8deb6`, `crates/isometry-system/data/monsters.json`,
  319,322 bytes, 334 records. Copied byte-for-byte from the pinned commit.
- Original authored TOML: the same revision's demo generator pack, 850 bytes.
- JSON scale fixture: 10,218,307 bytes, 10,688 records, created by repeating the
  original raw array contents 32 times. Duplicate identities make this a size-only
  synthetic workload; it is not an admissible game pack.
- TOML preservation fixture: the original pack plus comments and a foreign
  extension table, 1,115 bytes. Explicitly synthetic.
- Fixtures, SHA-256 hashes, exact supplier paths and revisions:
  [manifest](fixtures/manifest.json).
- REDox reference reviewed at `b60dd4d`; Mere's format policy checked at
  `5b3442ddd`. REDox itself was not executed. No .NET bridge or DOX format was added.
- Probe source, fixtures and receipts stay in this isolated testing directory.
  Product checkouts were not edited.

## Implementations

`serde_dom` fully materializes JSON values. `sonic_dom` uses Sonic's retained
mutable arena. Both support general object/array insertion and removal; the
Sonic structural-edit test also drops the original input before mutation.

`serde_typed_names` is a narrow borrowed extractor. It discards unknown fields
from its retained representation, so it is only an extraction baseline, not an
editable document candidate.

`raw_overlay` is a bounded prototype (`src/source_document.rs`, local only). It owns a shared
`Arc<str>`, stores source ranges for the root array's records, lazily caches name
spans/decoded escapes, and keeps a replacement overlay keyed by ordinal record
ID. Queries see edits immediately. Replacing the same field repeatedly leaves
one pending edit. Encoding copies untouched source spans and replaces only the
edited string tokens. There is no unsafe parsing or self-referential storage.

The prototype supports string-name replacement in a root array of objects. It
does not support structural edits, arbitrary paths, document-fenced IDs,
generation checks, concurrent readers/writers, or an undo journal. Ordinal IDs
are stable only within one retained document under its replacement-only model.

`toml_value` retains semantic values and reserializes them. `toml_edit` retains
formatting and comment structure. Replacement explicitly carries over the
value's decoration so a trailing comment survives.

## Correctness and persistence

- Six release tests pass: source ownership after caller drop, retained ID reads
  across edits, exact untouched bytes, escaped Unicode and out-of-order edits,
  malformed input/edit target handling, and JSON/TOML structural edits.
- Formatting and Clippy pass, including all targets and all features with
  warnings denied. [Clippy log](results/clippy.txt).
- [Verification](results/verification.json) contains 13 receipts. For both the
  real catalogue and an edge-case fixture, all three document candidates
  preserve semantic values and unknown fields after edits and reparsing.
- Source-backed JSON is byte-identical on a no-op encode. Replacement preserves
  unrelated numeric lexemes (`1.00e+2`), escapes, spacing, and nested unknown data.
- `toml_edit` preserves every unrelated line in the tested packs, including
  comments, unknown nested fields, and literal strings. `toml_value` preserves
  semantic unknown fields but loses the comments, as expected.
- Repeated edits: 2,000 replacements keep the final value correct; the source
  overlay still contains one pending edit.
- Edited JSON and TOML were written with `File::sync_all`, then read and checked
  in a distinct process. Writer PID 41193; reader PID 41194.
  [Fresh-process receipt](results/fresh-reopen.json). This is local file
  persistence evidence, not power-loss, native-host, or cross-platform qualification.

**Preserved validation boundary:** `RawValue` accepts the syntax of a foreign
string containing an unpaired `\uD800` escape, while full JSON value decoding
rejects it. The source prototype therefore does not prove complete scalar
semantic validation. A document tool may preserve foreign bytes, but admission
must still validate the application's typed data and canonical encoding. Editing
a malformed name itself is rejected when that name is decoded.

## Real JSON catalogue

| Operation | Backend | Median | Process median range | Speedup | Retained extra heap |
|---|---|---:|---:|---:|---:|
| `parse_select3` | `serde_dom` | 653.32 µs | 632.11–927.50 µs | 1.00× | 1553.03 KiB |
| `parse_select3` | `sonic_dom` | 96.82 µs | 94.73–106.48 µs | 6.75× | 823.31 KiB |
| `parse_select3` | `serde_typed_names` | 166.30 µs | 164.80–182.66 µs | 3.93× | 12.00 KiB |
| `parse_select3` | `raw_overlay` | 165.59 µs | 161.94–182.96 µs | 3.95× | 18.27 KiB |
| `parse_all_names` | `serde_dom` | 626.54 µs | 620.98–1125.78 µs | 1.00× | 1553.03 KiB |
| `parse_all_names` | `sonic_dom` | 103.53 µs | 99.94–224.96 µs | 6.05× | 823.31 KiB |
| `parse_all_names` | `serde_typed_names` | 168.70 µs | 164.69–252.42 µs | 3.71× | 12.00 KiB |
| `parse_all_names` | `raw_overlay` | 344.89 µs | 342.05–577.92 µs | 1.82× | 18.27 KiB |
| `parse_edit_encode` | `serde_dom` | 868.00 µs | 806.14–1174.54 µs | 1.00× | 1989.52 KiB |
| `parse_edit_encode` | `sonic_dom` | 257.07 µs | 234.57–296.74 µs | 3.38× | 1396.45 KiB |
| `parse_edit_encode` | `raw_overlay` | 181.19 µs | 164.52–221.76 µs | 4.79× | 330.47 KiB |

## Retained JSON operations

`retained_read1000` is 1,000 queries of three names (3,000 field reads).
`retained_edit100` is 100 short-name replacements on the same retained field.
`retained_encode` emits the whole edited document, without parsing again.

| Operation | Backend | Median | Process median range | Speedup | Retained extra heap |
|---|---|---:|---:|---:|---:|
| `retained_read1000` | `serde_dom` | 23.56 µs | 23.13–42.56 µs | 1.00× | 0.00 KiB |
| `retained_read1000` | `sonic_dom` | 45.70 µs | 44.68–96.23 µs | 0.52× | 0.00 KiB |
| `retained_read1000` | `raw_overlay` | 12.06 µs | 11.51–12.21 µs | 1.95× | 0.00 KiB |
| `retained_edit100` | `serde_dom` | 4.81 µs | 4.50–5.88 µs | 1.00× | 0.00 KiB |
| `retained_edit100` | `sonic_dom` | 6.96 µs | 6.88–9.64 µs | 0.69× | 0.00 KiB |
| `retained_edit100` | `raw_overlay` | 2.09 µs | 2.07–2.60 µs | 2.30× | 0.00 KiB |
| `retained_encode` | `serde_dom` | 189.02 µs | 181.85–294.99 µs | 1.00× | 436.50 KiB |
| `retained_encode` | `sonic_dom` | 143.52 µs | 136.71–195.13 µs | 1.32× | 567.00 KiB |
| `retained_encode` | `raw_overlay` | 4.56 µs | 4.55–8.13 µs | 41.47× | 311.83 KiB |

Retained heap after long replacements is bounded in this tested session:

- `serde_dom`: 1,590,298 → 1,590,327 retained extra bytes (+29) after 2,000 long name replacements.
- `sonic_dom`: 843,066 → 849,443 retained extra bytes (+6,377) after 2,000 long name replacements.
- `raw_overlay`: 18,704 → 19,113 retained extra bytes (+409) after 2,000 long name replacements.

The before/after measurements include different amounts of lazy cache and edit
metadata. They demonstrate bounded retention at 2,000 edits, not a universal
absence of growth for all structural workloads.

## JSON size-only expansion

| Operation | Backend | Median | Process median range | Speedup | Retained extra heap |
|---|---|---:|---:|---:|---:|
| `parse_select3` | `serde_dom` | 22602.54 µs | 20466.17–32016.33 µs | 1.00× | 49696.81 KiB |
| `parse_select3` | `sonic_dom` | 3497.15 µs | 3133.73–3776.42 µs | 6.46× | 26362.21 KiB |
| `parse_select3` | `serde_typed_names` | 5805.54 µs | 5281.25–6682.04 µs | 3.89× | 384.00 KiB |
| `parse_select3` | `raw_overlay` | 5485.12 µs | 5272.12–6988.83 µs | 4.12× | 584.50 KiB |
| `parse_all_names` | `serde_dom` | 23021.92 µs | 21227.00–28645.79 µs | 1.00× | 49696.81 KiB |
| `parse_all_names` | `sonic_dom` | 3615.19 µs | 3352.44–3675.67 µs | 6.37× | 26362.21 KiB |
| `parse_all_names` | `serde_typed_names` | 5801.38 µs | 5374.08–5820.79 µs | 3.97× | 384.00 KiB |
| `parse_all_names` | `raw_overlay` | 12070.46 µs | 10967.33–12150.29 µs | 1.91× | 584.50 KiB |
| `parse_edit_encode` | `serde_dom` | 31420.25 µs | 28197.62–31433.92 µs | 1.00× | 63664.82 KiB |
| `parse_edit_encode` | `sonic_dom` | 9000.00 µs | 8160.54–9135.38 µs | 3.49× | 44674.13 KiB |
| `parse_edit_encode` | `raw_overlay` | 5911.33 µs | 5413.12–6806.08 µs | 5.32× | 10563.69 KiB |

## TOML packs

The real authored pack is small. Preservation and a usable mutation API matter
more here than a parse-throughput win. The annotated fixture makes comment and
unknown-field behavior observable.

| Operation | Backend | Median | Process median range | Speedup | Retained extra heap |
|---|---|---:|---:|---:|---:|
| `parse_query` | `toml_value` | 8.24 µs | 7.69–8.80 µs | 1.00× | 7.82 KiB |
| `parse_query` | `toml_edit` | 13.09 µs | 11.91–13.36 µs | 0.63× | 21.59 KiB |
| `retained_edit100` | `toml_value` | 2.90 µs | 2.80–2.99 µs | 1.00× | 0.00 KiB |
| `retained_edit100` | `toml_edit` | 6.51 µs | 6.44–6.70 µs | 0.45× | 0.00 KiB |
| `parse_edit_encode` | `toml_value` | 14.24 µs | 13.70–14.54 µs | 1.00× | 8.76 KiB |
| `parse_edit_encode` | `toml_edit` | 16.46 µs | 15.57–16.54 µs | 0.87× | 22.56 KiB |

| Operation | Backend | Median | Process median range | Speedup | Retained extra heap |
|---|---|---:|---:|---:|---:|
| `parse_query` | `toml_value` | 9.39 µs | 9.05–9.43 µs | 1.00× | 9.87 KiB |
| `parse_query` | `toml_edit` | 14.64 µs | 14.13–14.78 µs | 0.64× | 25.60 KiB |
| `retained_edit100` | `toml_value` | 2.99 µs | 2.79–5.72 µs | 1.00× | 0.00 KiB |
| `retained_edit100` | `toml_edit` | 8.69 µs | 8.20–19.52 µs | 0.34× | 0.00 KiB |
| `parse_edit_encode` | `toml_value` | 17.16 µs | 16.15–30.08 µs | 1.00× | 11.77 KiB |
| `parse_edit_encode` | `toml_edit` | 19.38 µs | 18.41–35.42 µs | 0.89× | 27.64 KiB |

## Measurement limits

Host: Apple M4, arm64 macOS 26.5.1, Rust 1.97.1. Release builds use
`-C target-cpu=native`. Dependencies are locked to the versions already found in
Mere's lockfile: serde 1.0.229, serde_json 1.0.151, sonic-rs 0.5.10, toml 1.1.6,
toml_edit 0.25.15. Transitive versions are pinned by this probe's Cargo.lock.

The table reports the median of three process medians. Each process takes 15
calibrated samples per operation, after warm-up. Raw samples and per-process
ranges remain in the receipts. Timing includes destruction of returned results;
retained operations exclude initial parsing and preparation. No disk I/O is
included in timing. This is a bounded local microbenchmark under normal machine
load, without isolated CPU scheduling or a universal performance claim.

Memory uses requested-size allocator counters in a separate executable. The
timing executable uses the uninstrumented System allocator. Allocation metrics
include total requested bytes, allocation/reallocation calls, peak additional
live bytes, and live bytes retained at the end of each operation before its
result is dropped. These are incremental heap measures, not process RSS. They
exclude allocator metadata, OS mappings, and source buffers already resident
before the operation. Parser caches are warmed. `raw_overlay` must keep its
shared source alive; full DOM candidates can release the original input.

The initial overlay reserved only the original source length when encoding.
On the 32× fixture, a slightly longer name triggered a doubled output buffer.
The final implementation pre-encodes changed tokens and reserves the exact
result length. The original measurement and source are preserved in
[pre-fix summary](results/pre-capacity-fix-summary.json) and
pre-fix source (`results/pre-capacity-fix-source.rs`, local only); the tables above use the
corrected implementation.

## Resource recommendation

A caller-facing document resource could own source bytes and a revision, expose
document-fenced handles, stage replacements/insertion/removal, offer read views,
and encode through a format-aware backend. An edit receipt should identify its
base revision and dirty ranges/paths. Stable identity, stale-handle behavior,
undo, compaction, and thread ownership require explicit decisions before general
reuse; the prototype makes no claims about those.

Start the resource at a concrete inspector or pack-editor consumer. TOML should
retain its syntax and comments through `toml_edit`; JSON tooling can choose a
Sonic arena when it needs general mutation, or source overlays when untouched
bytes must survive exactly. Typed imports can continue directly through Serde.
Only factor common API and lifecycle operations that consumers actually share.

Canonical domain identity remains based on the validated parsed form or its
canonical encoding, as required by the current format policy. An editable syntax
resource does not acquire authority over world state, grants, or replication.
Persist authored files in their established text format; retain framed postcard
and deterministic CBOR in their existing roles.

## Reproduction and artifacts

Run `python3 run.py` from this directory. It prepares exact fixtures, checks
formatting, runs release tests and verification, builds separate timing/memory
executables, performs three timing runs, and records fresh-process reopening.
The source commit must be available in `../../repos/isocosm`; `ISOCOSM_REPO`
can point to another checkout containing it. The script needs Cargo and Python 3.

- [Machine-readable summary](results/summary.json), including fixture and
  source/binary hashes.
- [Environment](results/environment.json).
- [Timing run 1](results/timings-1.jsonl), [run 2](results/timings-2.jsonl),
  [run 3](results/timings-3.jsonl), and [allocations](results/allocations.jsonl).
- [Edited TOML](results/edited-pack.toml) and edited JSON (`results/edited-monsters.json`, local only).
- [Tests](results/tests.txt), [verification](results/verification.json),
  [write receipt](results/persistence-write.json), [fresh reopening](results/fresh-reopen.json).

Primary references: [REDox architecture](https://github.com/CAPCOM-TD-OSS/REDox#5-architecture),
[Sonic API](https://docs.rs/sonic-rs/0.5.10/sonic_rs/),
[toml_edit API](https://docs.rs/toml_edit/0.25.15/toml_edit/),
[RawValue API](https://docs.rs/serde_json/1.0.151/serde_json/value/struct.RawValue.html).
