# Retained document evidence log

**Date:** 2026-10-19. Dated evidence for the
[retained document resource brief](../../../../2026-10-14_retained_document_resource_brief.md),
moved out of the brief on 2026-10-19. The wording is unchanged except for
links and notes on which artifacts were copied. Each section records one experiment or continuation as of its own date;
later sections supersede earlier limits where they say so. Copied reports and
receipts sit beside this log, one folder per M4-local testing directory; the
[receipts index](README.md) lists what was copied and what stays local.

## Precursor probe — 2026-10-13

The completed [October 13 probe](redox-document-20261013/REPORT.md) measured Sonic, `serde_json`,
`toml_edit`, and the source overlay on the real 334-record monster catalogue
and the demo pack. Its overlay preserved untouched bytes exactly, `toml_edit`
preserved unrelated lines and comments, and the
[fresh-process reopening](redox-document-20261013/results/fresh-reopen.json) passed. Those results
support the resource concept; they are not Rowan implementation or
performance evidence.

## Parser evaluation — 2026-10-15

The
[comparison report](retained-parser-20261015/REPORT.md)
and [results](retained-parser-20261015/results/summary.json)
record 101 inherited unit tests passing for each backend, three probe tests,
3,075 UTF-8-safe prefixes, and 360 deterministic edits. Real pack variants
produce matching trees, diagnostics and DOM values, with exact text retention
and fresh-process reopen. Both default-feature libraries and all optional
features compile. The port changes no Rust source. Neither backend accepts the
four TOML 1.1 spot checks: `\e`, `\xHH`, multiline inline tables, and omitted
time seconds. These tests do not certify full TOML 1.0 conformance.

Full reparsing uses a fresh builder cache. Comparing the original annotated
snapshot with the final edited snapshot found zero shared green nodes out of
152 final nodes in both backends. Immutable snapshots work, but cross-revision
sharing is not supplied by this parse entry point. Local parser timings and
allocation counts gave no compelling reason to prefer the older Rowan solely
to avoid the manifest adaptation.

## Retained-document proof — 2026-10-16

The isolated
[report](retained-document-20261016/REPORT.md),
[syntax/save policy](retained-document-20261016/SYNTAX.md),
and [machine-readable results](retained-document-20261016/results/summary.json)
record an implemented resource and bounded pack consumer. The real
[demo pack](retained-document-20261016/fixtures/demo-pack.toml)
and its synthetic
[annotated variant](retained-document-20261016/fixtures/annotated-pack.toml)
retain their pinned fixture hashes. The consumer uses an unchanged copy of
Isocosm's manifest/validator at `20cb030fcc982db8fc89fa557918b8de5961543c`, with
a probe-local, text-only `GenValue` stand-in. It does not link product code,
load scripts/assets, or exercise the full generator ABI or product admission.

The supported dialect, `probe-toml10-pack-v1`, admits TOML 1.0-compatible
strings, integers, booleans, arrays and tables only when taplo and pinned
`toml` 0.8.23 agree on parsed values and typed validation succeeds. Floats,
date/time values, and unsupported TOML 1.1 forms block lowering but remain
retained, saveable drafts. Invalid UTF-8 and BOM input are rejected; source
size is limited to 2 MiB and lexical bracket/brace nesting to 64. Source hashes
use SHA-256. The proof domain hash uses a version-prefixed, recursively
key-sorted JSON encoding of the validated parsed value, including unknown
properties; arrays retain order and schema defaults are not materialized.
This local encoding does not change the stack's saved-data or wire policy.

Ten release tests (four copied-validator tests and six resource integration
tests), formatting, Clippy, standalone verification, and separate-process
reopening passed. They demonstrate:

- replacement, insertion and removal under `[[generators.lock_presets]]`, with
  correct ownership by the first generator;
- preserved comments, foreign content, literal strings, Unicode and CRLF;
- incomplete drafts with revision-stamped diagnostics and refused admission;
- unchanged old snapshots; stale-edit, wrong-document/instance, and stale
  same-document handle rejection;
- undo as a fresh revision without reviving handles or reversing admitted state;
- valid UTF-8 byte ranges and multi-edit receipts, with overlap rejection;
- typed manifest validation and negative controls for duplicate identities,
  duplicate TOML keys, missing names and unsafe paths;
- exact-byte save/reopen, stable domain hashes after reformatting/comment edits,
  changed hashes after opaque extension edits, worker snapshot analysis, and
  bounded history release.

The save backend serializes cooperating writers with a per-path lease file,
checks disk hashes, syncs a same-directory temporary file, and renames it over
the target. Completion records the revision actually saved; newer different
text stays dirty. Tests cover another cooperating resource, external changes
before/during save, cancellation, wrong-document tickets, and injected failure
before replacement. Conflicts require explicit reopen; there is no merge or
forced overwrite. The final hash-check/rename gap remains for uncooperative
writers. Directory fsync/crash durability, stale-lease recovery, metadata
preservation, path aliases and symlinks are not implemented or qualified.

Requested heap measurements include retained text, trees, hashes, diagnostics,
receipts and bookkeeping; fresh parser caches are dropped before publication:

| Total retained revisions | Retained bytes | After history release | After document drop |
|---:|---:|---:|---:|
| 2 | 26,796 | 13,718 | 0 |
| 17 | 224,742 | 15,254 | 0 |
| 65 | 859,206 | 21,398 | 0 |

History release keeps the current revision and some bookkeeping capacity.
These are requested allocation sizes, not RSS. Full reparsing still gives
linear retained-history growth; no incremental parse, direct structural edit,
or persistent cache optimization was implemented.

Three local M4 release timing processes measured edit publication separately
from lowering/validation/hashing. For the 850-byte demo pack, process-mean
medians were 102.42 and 223.62 microseconds respectively; for the 1,115-byte
annotated pack, 251.22 and 655.05 microseconds. The synthetic 4,096-key table
had substantially higher composite lowering cost. Raw samples show host-load
variation, so these are bounded observations rather than latency guarantees
or a like-for-like comparison with the earlier parser-only benchmark. The
subsequent stage profile and paired batching experiment below identify and
reduce the dominant wide-table construction work.

## Wide-table profile and bounded optimization — experiments 2026-10-16; recorded 2026-10-17

The
[stage profile](retained-lowering-profile-20261016/REPORT.md)
measured DOM construction at 116.303 ms of a 124.426 ms lowering pass, about
92.8% on the 4,096-key fixture. Separate counters and source inspection identify
repeated cloning of accumulated table entries: each `Shared::update` loads an
Arc still held by its slot, then `Arc::make_mut` clones the entry map/vector
before the next insertion. The 8,386,657 counted vector entry pairs equal
`4096 * 4095 / 2 + 97`; the fixed 97-pair addition comes from pack scaffolding.
Validation and JSON conversion add no further entry-container clones in this
valid corpus. These counts measure vector pairs, not allocation bytes or deep
syntax-tree copying; each clone also copies the lookup map.

The isolated
[batching report](retained-dom-batch-20261016/REPORT.md),
[paired results](retained-dom-batch-20261016/results/summary.json),
and [patch](retained-dom-batch-20261016/results/optimization.patch)
retain the candidate and unchanged control. The candidate batches consecutive
root entries and inline-table entries during private construction, stopping
at header boundaries. It reuses the original per-entry conflict body, verified
identical apart from indentation. Shared-container semantics and header/array
handling remain unchanged. The same 64,446-byte wide fixture gives these paired
local M4 release observations, with three fresh processes per backend and
backend/fixture order reversed in the middle pair:

| Measurement, 4,096 keys | Unchanged control | Batched candidate |
|---|---:|---:|
| DOM construction | 356.069 ms | 2.240 ms |
| Whole lowering, interleaved uninstrumented control | 380.106 ms | 14.472 ms |
| Vector entry pairs cloned during construction | 8,386,657 | 41 |

That is about 159x for DOM construction and 26.3x for whole lowering in these
paired runs. Counter binaries are separate from timing binaries. The optimized
copy count stays at 41 over 256, 512, 1,024, 2,048 and 4,096 keys, removing the
wide contiguous table's triangular term. Host-load variation prevents treating
these timings as portable guarantees or comparing separate experiment dates
as a speedup. Independent parsing and canonicalization account for more of the
remaining total; neither was changed.

Both libraries pass 101 inherited taplo tests each. Both resource backends pass
12 release tests each; the optimized counter-enabled configuration passes the
same 12. The direct differential corpus covers 20 named cases, 3,075 UTF-8-safe
prefixes, 360 deterministic edits and all five widths. It compares syntax and
semantic diagnostics, ordered DOM paths, primary/additional key source ranges,
node/descendant ranges, valid values and formatter output; paired runs also
require matching domain hashes. Duplicate keys, dotted-key merges, repeated
and scalar/table headers, inline-table conflicts and array-of-table ownership
are explicit controls. Formatting and probe Clippy pass. This is bounded
agreement, not exhaustive equivalence or full grammar qualification.

The isolated experiment remains the control/equivalence evidence for the
batched backend adopted by the private Isocosm consumer below. Dotted/pseudo-table
merges and repeated visits through separate headers can still clone populated
containers; this does not establish global linearity. Root batching introduces
an O(N) temporary vector, and peak allocation/RSS was not measured. The private
product compatibility checks below do not replace wider consumer/platform
checks or memory qualification. This optimization changes DOM construction
only: full-reparse syntax history still has the sharing/retention limits
measured above.

## Private Isocosm integration — published 2026-10-17

[Isocosm commit `ecff5762`](https://github.com/merely-made/isocosm/commit/ecff57627cb8340cd1514432f84cda47de3d0c16)
lands an optional, app-local `pack-author` CLI in `isometry-system`. The
[owning plan](https://github.com/merely-made/isocosm/blob/ecff57627cb8340cd1514432f84cda47de3d0c16/design_docs/2026-10-17_private_pack_authoring_plan.md)
and [qualification receipt](https://github.com/merely-made/isocosm/blob/ecff57627cb8340cd1514432f84cda47de3d0c16/design_docs/receipts/2026-10-17_pack_authoring/qualification.json)
record the implementation and bounded checks. The receipt describes the
pre-commit source state; the linked commit records publication. The optional
MIT taplo fork pins Rowan 0.17.0 and retains the tested root/inline-table
batching, conflict checks, source associations and ordering. Experiment-only
counters are excluded. Normal runtime builds do not depend on this backend;
no shared library API is introduced.

This consumer uses the actual `ContentPackManifest`, validator and full
`GenValue` schema rather than the earlier copied manifest/text-only stand-in.
The existing loader, schema, runtime admission and session code are unchanged.
TOML 1.x and externally supplied JSON keep their existing typed decoding and
original saved format, including the refusal of two competing manifests.
Unsupported TOML 1.1 syntax produces an advisory source-index diagnostic when
the authoritative decoder succeeds; a complete syntax index is not claimed.
Unknown content remains in authored text. No probe-specific domain hash or
identity encoding is promoted into the product. Isocosm remains in development;
this integration does not add ownership to its active campaign-retirement work.

The CLI retains immutable snapshots, revision-fenced handles, expected-base
UTF-8 text-edit batches and receipts, diagnostics, undo as a fresh revision,
real typed checks, and explicit draft saving. Invalid drafts remain saveable;
checking or saving never admits a manifest into live application state.
History is limited to 32 revisions; authoring input is limited to 2 MiB and
lexical TOML bracket/brace depth to 64. Cooperating saves use a per-path lease,
source SHA-256 checks and same-directory synced temporary-file replacement.
Tests cover external-change and stale-edit refusal, a newly conflicting second
manifest, failed replacement, exact CRLF/Unicode source, Unix mode preservation,
and refusal to replace symlink input. Noncooperating writers still have the
hash-check/rename race; directory fsync, crash/stale-lease recovery, ACL/xattr
preservation and nonlocal filesystem behavior are not qualified.

The published verification records 68 feature-enabled package tests, 58
default-path tests and 101 inherited backend tests passing, plus all-feature
all-target compilation, Rust 1.97.1 compatibility, formatting and scoped
Clippy. Package-wide `-D warnings` encounters an existing loader lint; strict
package-wide lint success is not claimed. The real demo pack and annotated
variant exercise the private consumer. Subprocess tests save an invalid draft,
reopen and repair it in a fresh process, then load the result with the existing
loader and execute the real demo generator fixture. The 8 authoring unit tests
and 2 subprocess tests were rerun before publication. These are terminal and
loader checks, without native GUI acceptance or new product timing/memory
measurements. The fork is committed as a local optional dependency; publishing
a tool distribution still requires separately qualifying that fork's packaging.

## Private CLI workflow qualification — 2026-10-17

The
[workflow report](pack-author-workflow-20261017/REPORT.md) and
[reproduction receipt](pack-author-workflow-20261017/reproduction/qualification.json)
record two successful runs against Isocosm
[`285821ad`](https://github.com/merely-made/isocosm/commit/285821ad6663e75a289ea047769aa9dfb226195c),
after fetching and fast-forwarding the clean checkout. The 170 KB
command/response transcript (`reproduction/transcript.json`) stays local.
Each run uses 45 JSON-line
commands across five fresh CLI processes on a scratch copy of the real demo
pack with its existing annotated manifest. The scripted caller uses public CLI
responses only: an inspected value range locates the generator-name edit;
revision-checked text edits insert a replacement preset and remove the old one.
Real typed validation confirms ownership under the intended generator and no
preset leakage. Comments and the complete foreign-extension tail survive.

An incomplete string is saved explicitly; the existing loader rejects that
draft. A fresh authoring process reopens the exact bytes, repairs the quote,
checks and saves, then the existing loader executes
`demo:forge_item fixtures/forge_item.json` successfully. A separate simulated
editor changes disk while an authoring draft is pending: save refuses the
external change, disk keeps the external text, and `read` retains the pending
draft. The caller exports that draft before quitting, explicitly reopens the
external version, reapplies the author change, and saves. A final fresh process
confirms exact bytes and clean state; the real fixture passes again. Checking
and saving remain separate from deliberate loader/fixture execution and report
`admitted:false`; no live application session is mutated.

No workflow blocker was found, so no product code or semantic editing command
was added. Original pack files and unchanged scratch assets are hash-checked;
the reproduction receipt also identifies the runner and built executables.
Caller orchestration of offsets/TOML ownership and explicit pending-draft export
remain concrete usability friction. This qualifies one scripted task, not
general semantic editing, automatic merge, durable session recovery, GUI use,
new performance/memory behavior or crash safety. The runner, transcript,
scratch packs and intermediate manifests stay in local testing; the report,
receipts and final manifest were copied here on 2026-10-19. This qualification
preceded the shared proposal continuation below.

## Shared proposal continuation — 2026-10-18

Inspection found a concrete
second consumer in Knot's source-authored local links. The user authorized the
[source-edit implementation plan](../../../../2026-10-18_source_edit_proposal_plan.md): a
small parser-neutral proposal, preview, revision/instance check and candidate
receipt, adopted by both hosts. This changes the earlier deferral decision.
Knot retains its one source buffer, undo/IME and file authority; Isocosm keeps
its format/schema checks and explicit save. The full retained-document session
API is not being extracted. `mere-source-edit` owns only validated text-edit
proposals, exact-source/instance/revision checks, previews and candidate receipts.
The core passes five tests and strict Clippy on the working toolchain, plus
Rust 1.97.1 tests. Isocosm passes 74 feature/58 default tests and its unchanged
59-command/five-process workflow. Knot passes 51 default/64 engine document
tests, ten link-model cases, four two-file UI-harness cases, one workspace
menu/focus case and explicit child-process reopening. Existing ignored cases
and strict test-Clippy baseline failures remain documented in the owning plan.
Knot keeps one editor buffer and one-step selection-edit undo; it refuses
multi-range application until it has an undo transaction. No parser is imposed
on Knot and no shared write authority is introduced.

The [qualification report](shared-source-edit-20261018/REPORT.md)
records exact source/check/override identities. Both consumer qualifications
use an explicit local Cargo patch for only the new supplier; existing supplier
versions are unchanged. Normal remote-only resolution, publication and native
GUI review are separate and remain unqualified.

## Duplication-removal continuation — 2026-10-18

One immutable
`SourceSnapshot` now retains and checks the source observation used to construct
proposals. Knot removes its form-owned address/text/stamp triple in favor of
that snapshot, including the handwritten source equality guards. It retains
host authority/path/target/selection/IME checks and its existing editor. Isocosm
routes undo through the same shared edit/publication path, consuming the checked
result revision and ranges rather than building either separately. Six core
tests, Isocosm 75 feature/58 default tests and unchanged 59-command workflow,
Knot 53 default/66 engine tests, 11 link-model cases, four two-file harness cases
and two child reopens pass. Existing ignored/lint cases stay separate. The
[deduplication report](source-edit-dedup-20261018/REPORT.md)
records the changed ownership and exact local-override qualification. This
removes duplicated state/checks, not necessarily total lines: extracted Knot
helpers and regression coverage grow while the form itself shrinks. Supplier
publication and normal remote-only receiving checks remain outstanding.

## Publication continuation — 2026-10-18

The supplier is published at
[Mere `70deef2c8`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f).
Its six tests, strict all-target Clippy and Rust 1.97.1 tests pass after the
fetched-main rebase. Git-only consumer receiving qualification is recorded in
the [publication report](source-edit-publication-20261018/REPORT.md).
[Isocosm `276157b7`](https://github.com/merely-made/isocosm/commit/276157b776c59394923550b0b42d543b317210d9)
and [Knot `d2d5c32`](https://github.com/merely-made/knot-editor/commit/d2d5c32942d40af47223790ed3e16bc88fa59aa6)
pass the receiving suites and workflows above without local overrides. Isocosm
locks the canonical main source; Knot selects only the new contract by its exact
published revision, avoiding unrelated rendering-family updates. Both preserve
all existing supplier versions/revisions.

## Isocosm continuation — 2026-10-19

Isocosm main [`2389818c`](https://github.com/merely-made/isocosm/commit/2389818c) lands five step commits on `c3ad4b97`,
then merges origin/main `028a5ef1`. The owning record is the P8 section of
Isocosm's `design_docs/2026-10-17_private_pack_authoring_plan.md`, with receipts in
`design_docs/receipts/2026-10-19_pack_authoring/`. A fresh clone of pushed
main, checked again by the coordinating session on 2026-10-20, passes 85
feature-enabled tests (two ignored helpers) and 58 default tests. The fork
branch passes 107 tests from a fresh clone.

**Stale save lease** ([`80137113`](https://github.com/merely-made/isocosm/commit/80137113)). The lease records PID, host
and source. A lease left by a dead process on the same host is reclaimed, along
with that saver's temporary file. A live holder, another host, or an old-format
lease still refuses. Tests kill a real saver mid-save, then save successfully;
a live holder still blocks. A dead saver's PID reused by a live process, for
example after a reboot, still needs manual removal.

**History memory** ([`8d6499c1`](https://github.com/merely-made/isocosm/commit/8d6499c1)). Three modes behind `--history`,
plus a 64 MiB byte budget alongside the 32-revision cap. Requested-heap retained
bytes on the annotated pack were identical across three release runs:

| Revisions | full | text | cached | 2026-10-16 baseline |
|---:|---:|---:|---:|---:|
| 2 | 26,632 | 14,896 | 23,424 | 26,796 |
| 17 | 222,584 | 34,928 | 145,672 | 224,742 |
| 65 | 849,544 | 98,944 | 537,240 | 859,206 |

After history release, 13,608 bytes remain in every mode; after document drop,
0. The cached mode shares 90 of 152 green nodes and 323 of 324 tokens across
revisions; full and text share none. Median edit publication, in microseconds
(full / text / cached): real pack 69.18 / 68.95 / 65.11; annotated pack
80.57 / 81.22 / 75.57; 4,096 keys 5,983 / 6,221 / 5,249; 65,536 keys
113,429 / 120,652 / 98,521. At 65,536 keys the byte budget keeps 3 revisions in
full and cached modes and 32 in text mode.

The default is **text**. It retains 88% less than full at 65 revisions, publishes
within noise of full, and no command reads an older revision's tree. Cached is
6–13% faster but retains 5.4 times as much as text. The `NodeCache` result
shows real structural sharing is available if a later consumer needs older
trees resident.

**Fork in its own repository** ([`b0f9988a`](https://github.com/merely-made/isocosm/commit/b0f9988a)). Mark confirmed the
name and public visibility. [merely-made/taplo](https://github.com/merely-made/taplo)
is a GitHub fork of tamasfe/taplo. Branch `merely/rowan-0.17` starts at upstream
tag `release-taplo-0.14.0` and adds four commits: Rowan 0.17
([`8ad7de4`](https://github.com/merely-made/taplo/commit/8ad7de4)), DOM batching ([`c316724`](https://github.com/merely-made/taplo/commit/c316724)),
`parse_with_cache` ([`118aae3`](https://github.com/merely-made/taplo/commit/118aae3)) and TOML 1.1
([`aa6d784`](https://github.com/merely-made/taplo/commit/aa6d784)). Isocosm pins `aa6d784` by exact revision; `vendor/taplo`
is removed. The fork passes 102 tests at `118aae3` (101 inherited plus one) and
107 at `aa6d784`. The earlier 3,460-document differential corpus matches the
unchanged control at both revisions.

**TOML 1.1** ([`5dd01ab8`](https://github.com/merely-made/isocosm/commit/5dd01ab8)). The fork accepts `\e`, `\xHH`,
multiline inline tables with comments and a trailing comma, and times without
seconds. TOML 1.1 is opt-in; TOML 1.0 stays the fork's default, so two inherited
tests that treat those inline-table forms as invalid still pass. Against
`toml` 1.1.8 and Isocosm's locked 1.1.6, 33 named cases and their 914 prefixes
give 167 accepted with equal values, 780 rejected by both, and no disagreements.
All four 2026-10-15 spot checks are accepted. The advisory partial-index
diagnostic is gone for valid 1.1 source. The fork's formatter preserves comments
and values in multiline inline tables but does not lay them out.

**Structure-aware commands** ([`286e690d`](https://github.com/merely-made/isocosm/commit/286e690d)). `insert_lock_preset`,
`replace_lock_preset`, `remove_lock_preset` and `set_generator_field` address
generators by id and presets by key. They publish through the existing
`mere-source-edit` proposal path with the same revision checks and receipts.
The unchanged 59-command workflow stops at one assertion, which expected TOML
1.1 source to block a preset proposal; step 4 removed that by design. With only
that assertion changed it passes, and 60 of its 64 records are identical to the
earlier transcript; the other four differ only in the removed diagnostic and
the refusal reason. A new workflow, 66 commands across five processes, passes
without caller byte offsets.

**Toolchain caveat.** Since `c3ad4b97`, cargo 1.97.1 cannot resolve the Isocosm
workspace: Eidetic's chartulary manifest uses a cargo 1.98+ form while
declaring `rust-version` 1.97.1. The gates compiled and tested with rustc and
rustdoc 1.97.1, driven by cargo 1.99.1. Eidetic was not changed.
