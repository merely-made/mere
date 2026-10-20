> Copied 2026-10-19 from the M4-local `~/Code/testing/retained-document-20261016/REPORT.md`; see the [receipts index](../README.md).

# Retained-document proof — Rowan 0.17

**Date:** 2026-10-16. **Status:** isolated implementation and bounded acceptance;
not product integration or a shared API.

## Result

The evaluated manifest-only taplo/Rowan 0.17 port supports an isolated retained
document lifecycle around the real Isocosm pack fixtures. The resource publishes
immutable text/tree/diagnostic revisions, fences handles and edits, retains
invalid drafts, lowers valid revisions through a copied real pack validator,
and saves/reopens exact authored bytes. A separate consumer decides admission.

The accepted dialect is explicit in [SYNTAX.md](SYNTAX.md): TOML 1.0-compatible
strings/integers/booleans/arrays/tables accepted by both taplo and `toml` 0.8.23,
with a text-only generator value ABI for this fixture proof. Floats, date/time
values, and the unsupported TOML 1.1 additions block lowering. Invalid source
remains saveable as a draft within the input limits. This is not full TOML
conformance certification.

## Source and consumer

The real 850-byte demo pack and synthetic 1,115-byte annotated variant are
copied unchanged from the preceding parser evaluation. The fixture provenance
pins Isocosm `e5f8deb60049e7622206ef2fa32cbe12e4bf822d`; hashes are checked on
rerun. The annotated fixture includes comments, a literal backslash string,
and an opaque foreign table.

The manifest/validator module is copied byte-for-byte from the Isocosm revision
in [provenance.json](results/provenance.json), with its four unit tests retained.
It checks format, identity, duplicate generators/presets, relative asset paths,
and other manifest rules. The full product `GenValue` ABI depends on wider
campaign types; this proof supplies only its text-shaped variant. The
consumer uses the actual copied manifest validator but does not load scripts,
resolve assets, execute generators, or change product admission/world state.

The taplo source is copied from the completed parser evaluation. It still
changes no Rust source, only the manifest dependency/package identity for Rowan
0.17. [parser-source-hashes.json](results/parser-source-hashes.json) checks the
retained source, and [parser-port.patch](results/parser-port.patch) records the
original adaptation. Both product repositories remain outside this build graph.

## Lifecycle evidence

The end-to-end scenario and focused tests demonstrate:

- Text replacement, insertion, and removal under `[[generators.lock_presets]]`;
  the inserted preset belongs to the first generator, not the following one.
- Exact preservation of comments, extension content, literal strings, Unicode,
  and CRLF. Multi-edit receipts carry base and resulting byte ranges, including
  offsets shifted by earlier edits.
- Full reparsing before publication, with emitted tree text checked against the
  authoritative source. Incomplete input produces revision-stamped diagnostics
  and blocks lowering, while the source remains a saveable draft.
- An unchanged earlier snapshot; expected-base rejection; wrong-document,
  same-name/different-instance, and stale same-document handle rejection.
- Undo as a fresh revision. Returning to identical text does not revive old
  handles and does not reverse the consumer's previously admitted changed pack.
- Correct typed lowering using the copied validator; negative controls for
  duplicate generator IDs, duplicate TOML keys, missing names, and unsafe paths.
  Schema/semantic failures carry the source revision, with coarse whole-document
  ranges where no precise mapping is implemented.
- Explicit unsupported-dialect diagnostics, invalid encoding/BOM rejection,
  byte-boundary rejection without publication, batch overlap/same-start
  rejection, source-size/nesting limits, and expired-history behavior.
- A snapshot passed to a worker and lowered there with its own navigation tree;
  compile-time `Send + Sync` checking of the retained snapshot type.
- Count-bounded history and receipts; retained snapshots survive eviction, and
  old snapshot allocation reachability ends when readers/history release them.

The standalone scenario saves annotated, edited, Unicode/CRLF, and incomplete
files. A separately launched process compares exact source hashes, diagnostic
counts, and available domain hashes after reopening.

See [verification.json](results/verification.json),
[fresh-reopen.json](results/fresh-reopen.json), and [tests.log](results/tests.log).

## Identity and admission

Source identity uses SHA-256 of exact bytes. Domain identity hashes a
version-prefixed, recursively key-sorted JSON encoding of the validated parsed
value. Unknown content is included: modifying an opaque extension changes the
hash; changing only comments or formatting does not. Integer base spellings
with equal values also have the same hash. Array order remains significant.
Defaults are not materialized. The precise bounded encoding and prefix are in
[SYNTAX.md](SYNTAX.md); this is not a new product or protocol canonical format.

The isolated consumer calls lowering before updating its admitted state.
Invalid revisions cannot replace that state. Document edits/undo do not call
admission implicitly. Document name/instance/revision stamps are local handles,
not durable domain IDs, grants, replication IDs, or persistent history metadata.

## Saving and external changes

Saving holds a per-path create-new lease file. It writes/syncs a temporary file
and renames it in the same directory. Disk hashes detect observed external
changes and refuse overwrite; explicit reopen is the reload policy. A save
finishing after a newer edit records the older saved revision and leaves the
different current text dirty. An older revision than the last saved one cannot
start a save, and another cooperating writer cannot acquire the active lease.

The negative controls cover concurrent edits while a ticket is held, a second
cooperating resource, an external write after ticket creation, an external
change before save begins, injected failure after temporary-file sync and before
rename, wrong-document tickets, and cancellation. Failed/cancelled saves remove
ordinary lease/temp files and do not advance the saved marker. External content
is preserved when a conflict is detected.

The final hash-check/rename gap remains for uncooperative writers. This is
not compare-and-swap, multi-host coordination, or crash-durability qualification.
Directory fsync, stale-lease recovery after process death, permissions/metadata
preservation, and alias/symlink handling are outside this backend. Tests use an
owned local testing directory. Those are concrete adoption limits rather than
properties supplied by Rowan.

## Measurement

The runner records edit publication separately from lowering/validation/domain
hashing, across three fresh processes on the local M4. Edits use full reparsing
and retain at most four history entries. The large fixture appends 4,096 keys
under `[vendor.wide]`, and measures beginning/middle/end edits separately; it
is a layout/size fixture, not a new product pack. Results are local process means
and medians, not portable performance guarantees.

Edit publication includes text search/replacement, preflight lexing, taplo
parsing, exact tree-text reconstruction, the independent TOML 1.0 parse/policy
check, source hashing, receipt construction, and history eviction. Lowering
includes DOM construction/validation, JSON conversion, a fresh independent
parse and value comparison, typed manifest validation, canonicalization, and
hashing. The composite timings do not attribute cost to a particular stage.

A separate allocation-meter binary retains 2, 17, and 65 total revisions
(initial plus 1, 16, or 64 edits), then releases history/receipts and drops the
document. Requested-byte counters include retained text, trees, diagnostics,
hashes, receipts, and history bookkeeping. Temporary builder caches are fresh
per parse and dropped before publication. Persistent cache reuse and structural
sharing optimization are not implemented; process RSS is not measured.

Numeric results are in [summary.json](results/summary.json),
[memory.json](results/memory.json), the three timing files, and the appended
measurement table below. Full reparsing still has linear retained-history
cost; the explicit history cap and release behavior are part of the proof.

## Reproduction and boundaries

Run `python3 run.py` here. The script verifies copied-source/fixture hashes,
checks formatting and Clippy, runs ten release tests (four inherited validator
unit tests and six resource integration tests), then executes verification,
fresh-process reopening, timings, and separate allocation metering. The root
`Cargo.lock`, [environment](results/environment.json), and dependency tree
pin the toolchain and closure. First runs may fetch dependencies; reproducing
the proof needs no product checkout. The parser's 101-test inherited suite was
already passed in the preceding unchanged-versus-port evaluation; this proof
adds document/consumer checks without changing parser source.

No shared API extraction, product code or documentation change, commit, push,
or product adoption is performed. Scripts/assets, full generator-value ABI,
world-state admission, product hashes/wire formats, durable revision history,
and multi-consumer lifecycle generalization remain outside the demonstrated
scope. This proof establishes the isolated lifecycle and one bounded pack
consumer, not a completed production resource.

## Recorded measurements

Median process means from three fresh release timing processes:

| Fixture / edit position | Edit publication | Lower / validate / hash |
|---|---:|---:|
| real / name | 102.42 µs | 223.62 µs |
| annotated / name | 251.22 µs | 655.05 µs |
| wide4096 / beginning | 37,797.46 µs | 462,259.55 µs |
| wide4096 / middle | 24,909.91 µs | 424,803.02 µs |
| wide4096 / end | 32,759.69 µs | 448,548.41 µs |

The wide-table lowering cost is material in this full validation pipeline.
These composite measurements do not establish whether DOM validation, the
independent parse, canonicalization, or another stage dominates; stage profiling
would precede optimization. Samples show host-load variation, so these local
measurements should not be treated as latency guarantees.

| Total retained revisions | Retained requested bytes | After history release | After document drop |
|---:|---:|---:|---:|
| 2 | 26,796 | 13,718 | 0 |
| 17 | 224,742 | 15,254 | 0 |
| 65 | 859,206 | 21,398 | 0 |

History release keeps the current revision and some allocated bookkeeping
capacity, including the cleared receipt vector's capacity. It releases old
snapshot references; it is not a claim that every allocation returns to the
one-revision baseline. Full document drop returns the measured increment to
zero. Counters report requested heap bytes rather than RSS or allocator metadata.

[isolation.json](results/isolation.json) records product status/HEAD checks before
and after this proof. Both working-tree statuses remained unchanged.

Mere HEAD advanced independently during the probe, from `cd795487b` to
`88a05e10f`; its working tree was clean at both checks. This session performed
only read operations in product repositories and did not reset, merge, commit,
or modify that concurrent work. Isocosm HEAD and status were unchanged.
