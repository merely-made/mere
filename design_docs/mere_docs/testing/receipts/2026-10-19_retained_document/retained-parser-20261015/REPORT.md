> Copied 2026-10-19 from the M4-local `~/Code/testing/retained-parser-20261015/REPORT.md`; see the [receipts index](../README.md).

# Bounded retained TOML parser evaluation

**Date:** 2026-10-15 (America/New_York). **Status:** completed, isolated probe.

## Decision

The Rowan 0.17 option is a manifest-only compatibility adaptation, not a
lexer/grammar rewrite. Keep it as the preferred candidate for the next isolated
retained-document proof when a Rowan 0.17 backend is desired. The tested library
compiles and passes its inherited tests without any Rust source changes.
There is no measured behavior or allocation advantage to unchanged taplo that
would justify keeping the older Rowan solely to avoid this adaptation.

Neither option qualifies as a TOML 1.1 parser. Both reject all four tested 1.1
additions. Use the candidate for the real pack's demonstrated syntax only with
an explicit accepted-language policy. If TOML 1.1 is required, grammar support
or another parser needs a separate evaluation before adoption. This comparison
has not certified full TOML 1.0 conformance either.

No product code, product lockfiles, documentation, or the earlier probe was
edited. This report does not implement the retained-document resource.

## Inputs and versions

The crates.io API was queried during this run. The newest stable taplo library
is **0.14.0**, released 2025-05-22. Its dependency constraint is Rowan ^0.15.3;
this run resolves **0.15.19**, not 0.15.3. The newest stable Rowan is **0.17.0**;
0.18.0-alpha.1 is a prerelease and is excluded. Registry observations and UTC
check timestamps are in [releases.json](results/releases.json).

The exact taplo archive has SHA-256
`c221a50eef1a5493074f11ca1ed62bef28c05a4d925002944cc686b2e783a5b3`.
Its packaged VCS metadata names upstream commit
`15963fcb555af6bb543e79caf664c3daa110f2ad`. The entire extracted library is
retained, including DOM, formatter, utilities, and inherited tests. The baseline
uses that unchanged local source to make its inherited unit tests runnable;
it is not a modified baseline or a parser-only extraction.

The port changes three manifest lines: package name, library name, and Rowan
constraint. The first two permit side-by-side imports; only the third is the
functional dependency change. **Zero Rust source files differ.** The exact
[patch](results/port.patch) and [scope receipt](results/port-scope.json) are
retained. A future distribution would still own dependency updates, licensing,
and regression checks; this result establishes initial compatibility rather
than a promise that upstream upgrades will remain effortless.

Fixtures came from the preceding retained-document probe:

| Fixture | Bytes | Provenance |
|---|---:|---|
| `demo-pack.toml` | 850 | Real Isocosm demo generator pack at `e5f8deb60049e7622206ef2fa32cbe12e4bf822d`, `crates/isometry-system/examples/packs/demo/isometry-pack.toml` |
| `annotated-pack.toml` | 1,115 | Explicitly synthetic variant adding comments, a foreign extension table, and a literal string |

The original was reverified byte-for-byte with `git show` at the pinned
revision. Both fixture hashes match the earlier manifest; provenance is in
[fixtures/provenance.json](fixtures/provenance.json). These are authored pack
fixtures; the JSON monster catalogue is not part of this parser comparison.

## Behavioral checks

Both backends passed:

- Their **101 inherited taplo unit tests each**, including parser/invalid-input,
  formatter, and DOM rewrite tests. The benchmark harness and doctests were not
  used as conformance tests.
- All **3 probe tests**, including the complete differential verification.
- Sixteen named cases: real/annotated packs, Unicode and CRLF, incomplete
  strings/headers/inline tables, unexpected tokens, duplicate keys, name
  replacement, nested preset insertion/removal, a date-time control, and four
  TOML 1.1 spot checks.
- **3,075 prefixes** cut only at UTF-8 boundaries across real, annotated, and
  Unicode pack variants; **360 deterministic edits** inserting/removing text,
  punctuation, Unicode, line endings, and NUL characters.
- Identical source reconstruction, normalized node/token kinds and ranges,
  token text, syntax diagnostic messages/ranges, semantic error debug values,
  and JSON DOM values across the differential corpus. Syntax diagnostic bounds
  are checked against byte length and UTF-8 boundaries; token ranges cover all
  source bytes contiguously.
- Separate-process save/reopen of annotated, Unicode/CRLF, incomplete-string,
  and inserted-preset documents. Both emissions match before saving; a new
  process checks exact bytes and repeats the parser comparison.

For valid pack variants, the full DOM JSON value equals an independent parse
with `toml` 1.1.6. A probe-local typed projection of pack/generator/preset fields
also agrees. The inserted `mood` preset belongs to the first generator, the
second generator has no preset, and removal leaves the first without a preset.
The extension table and literal string survive unrelated edits. This projection
is not the product schema, semantic admission, canonical hashing, or a proof
that all TOML scalar representations lower identically. Date-time is an
acceptance control, not a full cross-library scalar-equivalence test.

Duplicate keys produce DOM validation errors with no syntax errors. Incomplete
input produces syntax diagnostics and exact retained text. The comparison
only constructs/serializes DOM values when syntax diagnostics are absent and
DOM validation succeeds; it does not exercise partial DOM recovery as an API.

See [verification.json](results/verification.json),
[fresh-reopen.json](results/fresh-reopen.json), and the test logs.

## Language coverage is the limiting result

The official [TOML 1.1 specification](https://toml.io/en/v1.1.0) permits these
forms; `toml` 1.1.6 accepts each tested case, while both taplo backends reject it:

| TOML 1.1 case | Both taplo versions |
|---|---|
| `value = "\e"` | Invalid escape sequence |
| `value = "\x41"` | Invalid escape sequence |
| Multiline inline table with comments/trailing comma | Syntax diagnostics |
| `value = 07:32` | Syntax diagnostics |

These are valid-source rejection findings, not failures of lossless retention:
both trees still reconstruct the input. Changing Rowan cannot add missing
lexer, escape-decoding, grammar, or DOM support. A future TOML 1.1 adaptation
must check all of those layers. The inherited suite and bounded edit corpus
are not a complete specification suite or hostile-input fuzz campaign.

## Retention and memory

Both preserve the original annotated snapshot while retaining 64 later edited
parses. Comparing the original tree with the final edited tree finds **0 shared
green nodes out of 152 final nodes** for either backend. The parser creates a
fresh builder/cache for each parse, so a full-reparse baseline does not deliver
cross-revision sharing merely by selecting Rowan.

A separately compiled allocation-meter binary measures live requested heap
bytes. The temporary edited source buffers have been dropped at measurement;
the preallocated history vector capacity is excluded from the increment.
Both backends report the same results:

| Retained revisions | Retained requested bytes | Incremental bytes after clearing history |
|---:|---:|---:|
| 1 | 11,664 | 0 |
| 16 | 186,624 | 0 |
| 64 | 746,496 | 0 |

At 64 revisions the incremental peak is 752,547 bytes for both. These are
requested allocation sizes, excluding allocator metadata and RSS. Retaining
an authoritative source buffer, resource receipts, or additional analyses would
add costs. Persistent cache reuse, cache reset policies, incremental parsing,
and structural edit optimization were not implemented. The exposed `parse`
entry point does not accept a caller-owned cache.

See [memory.json](results/memory.json) and the snapshot section of verification.

## Timing

Local Apple M4/macOS, Rust 1.97.1, release profile. Three fresh timing processes,
with backend order reversed in the middle process; no allocation meter in the
timing binary. Values below are medians of process means, in microseconds.
These observations do not establish a statistically significant speed difference.

| Fixture | Unchanged parse | Rowan 0.17 parse | Unchanged edit/reparse/emit | Rowan 0.17 edit/reparse/emit |
|---|---:|---:|---:|---:|
| Real demo | 10.95 | 10.99 | 25.00 | 25.65 |
| Annotated | 13.07 | 12.93 | 29.29 | 29.85 |
| Synthetic 4,096-key table | 1,484.97 | 1,487.81 | 2,672.90 | 2,701.38 |

The large fixture appends a flat `[wide]` table to the annotated pack. It is a
size/layout stress fixture, not an admissible product pack. Parsing includes
result destruction. The combined edit measurement includes allocating edited
source, full parsing, reconstructing text, and destroying the result. Emission
is also measured separately. Real fixtures use 2,000 iterations per process;
the large fixture uses 40.

Beginning/middle/end edits in the wide table are measured separately as text
replacement plus full reparse. This is a baseline for later optimization,
not a direct measurement of green-node ancestor rebuilding. Raw process
measurements, min/max, and samples are in `timing-*.json` and
[summary.json](results/summary.json). The retained
[environment](results/environment.json) and [dependency tree](results/dependency-tree.txt)
identify the exact runtime and dependency closure.

## Compatibility and maintenance conclusion

Both default-feature libraries build; all optional features compile for both.
The probe passes formatting and Clippy with warnings denied. Both local taplo
sources emit the same pre-existing unused `FormattedItem::syntax` warning;
upstream source was not altered to hide it. No Rust source port was necessary
for parser, DOM, formatter, or inherited tests. We did not run external taplo
CLI/editor consumers, cross-platform builds, an MSRV matrix, or a full TOML
conformance/fuzz suite. Rowan types remain private to this experiment; no
product compatibility or integration claim follows.

The next document slice can use the manifest-only Rowan 0.17 adaptation as its
baseline, provided its supported TOML syntax is explicit. Retain immutable
snapshots and start with text edits/full reparsing as planned, with memory
limits reflecting linear history growth in this implementation. Do not make
persistent sharing a requirement met by this parser without further work.
If the application requires TOML 1.1 now, neither candidate is ready; that
language-support decision comes before product adoption.

## Reproduction and scope

Run `python3 run.py` here. It validates fixture and archive hashes and ensures
that upstream source is unchanged and the port differs only by the recorded
manifest edits. It then runs focused checks, both inherited suites, three
timing processes, separate allocation metering, and fresh-process reopening.
The root `Cargo.lock` is retained and commands use `--locked`. First runs may
need network access to fetch dependencies. No product repository is needed to
rerun the retained fixture comparison.

[isolation.json](results/isolation.json) records unchanged Mere and Isocosm
working-tree statuses. No shared documentation/index update, product code,
commit, push, or production parser adoption was performed.
