# Retained document resource brief

**Date:** 2026-10-14. **Revised:** 2026-10-17. **Status:** Rowan remains the
working direction. The parser comparison, isolated document/pack proof, and
bounded DOM batching experiment pass. Isocosm has published a private optional
pack-authoring CLI with the batched backend and real manifest/full `GenValue`
schema. Its real-pack CLI workflow passes at `285821ad` without product fixes.
Durable history and a shared resource API remain deferred. The earlier
proofs stay in testing; the private consumer lives in Isocosm.

Use a retained document resource to inspect and revise authored data while
preserving its text, comments, unknown content, and earlier revisions. The
resource owns document identity, revision history, diagnostics, and edit
receipts. Rowan supplies the syntax tree. Format parsers and typed views
remain specific to the application and format.

**Why Rowan for TOML when `toml_edit` already preserves comments.**
`toml_edit` does not expose a retained syntax tree for a document that fails
parsing. A resource can still retain that source, its parse error, and earlier
revisions. Syntax-aware editing of incomplete input requires an error-tolerant
parser. Rowan supplies the immutable tree representation; the TOML parser must
preserve all input and produce diagnostics. In the checked API,
`Document::parse` and `FromStr` for `DocumentMut` return a document or a
`TomlError`. Keep `toml_edit` useful for parseable documents while proving the
Rowan consumer; the private Isocosm integration below is the first product
consumer.

## Rowan revisions

Rowan's immutable green tree permits unchanged subtrees to be shared between
revisions. Rowan 0.17.0 (2026-08-02) is immutable only: the in-place mutation
API of 0.16 (`clone_for_update`, `splice_children`, `detach` on syntax nodes)
was removed, so a retained root cannot be changed under a reader.
`SyntaxNode::replace_with` asserts that the replacement has the same kind and
returns a new green root, rebuilding ancestors on the path to the root while
sharing unchanged subtrees. Each rebuilt parent processes its child list, so
edit cost depends on the widths of those ancestors as well as tree depth.
Insertion and removal go through the parent's green node
(`insert_child`, `remove_child`, `splice_children`), followed by
`replace_with` on that parent. Retaining the old root preserves the previous
syntax revision. This provides a foundation for previews, document undo, and
analysis of a fixed revision. The isolated proof now exercises publication and
count-bounded history; product retention requirements remain open.
[Rowan replacement API](https://docs.rs/rowan/0.17.0/rowan/api/struct.SyntaxNode.html#method.replace_with).

**Revision consistency.** Every published revision has one authoritative
authored text. Its syntax tree and diagnostics describe that exact text.
Direct structural edits must preserve equivalence with parsing the resulting
text, or reparse before publication. Green-tree splicing alone does not
establish this: token boundaries, separators, and error recovery can change.

**Initial edit path.** Start with text edits followed by a full reparse.
Syntax-aware commands, such as inserting a generator, compute text edits from
the current tree and use the same publication path. Full reparsing does not
guarantee sharing across revisions; measure sharing separately. Incremental
reparsing and direct green-tree splicing are later candidates if measurements
justify them and equivalence checks pass. A receipt names both coordinate
spaces: base-revision ranges for what was replaced and result-revision ranges
for what was produced.

The first slice accepts UTF-8 documents and preserves their bytes, including
line endings, without normalization. Reject unsupported encodings explicitly.
Edit and diagnostic ranges use UTF-8 byte offsets; edit endpoints must be code
point boundaries. An editor using another coordinate system converts at its
boundary. The proof sorts batch edits in base-revision coordinates and refuses
overlaps and same-start edits, including ambiguous same-position insertions.

**Interning and memory.** Only the green-node builder interns through
`NodeCache`; structural edits allocate fresh nodes outside it. `NodeCache` is
a plain hash map with no eviction API, so one long-lived cache shared across
reparses can retain new entries until it is dropped. Evaluate document-scoped
cache reuse against fresh caches; define a bounded reset policy if reuse is
selected. Dropping history alone cannot release entries retained by a cache.
Retained-memory measurements must include
the cache as well as retained roots.

**Workers.** Green nodes are `Arc`-backed and `Send + Sync`. Red
`SyntaxNode`s use a non-atomic reference count and are neither. A worker
receives a green root and builds its own navigation tree, which is cheap.
This answers most of the worker-ownership question without a second library.

**Version choice.** The October 15 comparison passed for taplo 0.14 unchanged
(resolved Rowan 0.15.19) and the same library on Rowan 0.17.0. The port changes
only the dependency and package/library names for side-by-side evaluation;
zero Rust source changes were needed. Use the 0.17 port for the isolated proof.
The registry check found these to be the latest stable library releases on
that date; this is a dated observation, not an ongoing version check.
Rowan 0.18.0-alpha.1 (2026-09-14) attaches leading
and trailing trivia to tokens, so comments and whitespace move from sibling
tokens into the token they decorate, and `index()` becomes optional. That
changes where a parser places comments and what an edit carries with it.
Treat comment ownership as a parser decision to revisit at 0.18.

## Editing and admission

**Editing and admission have different boundaries.** A document revision may
contain incomplete or invalid text; its parser should preserve that text and
attach diagnostics. Saving a draft preserves authored bytes. Lowering a
revision into typed application data requires schema and semantic validation,
and the owning application decides whether to admit the result. A syntax
resource does not gain authority over world state, grants, or replication.
Document undo after admission reverts text, not admitted state; the
application must make that visible.

The proposed resource contract is:

- Open and retain a document with an explicit format and document identity.
- Read or snapshot a named revision without changing it.
- Apply an edit against an expected base revision; reject stale bases and
  publish a new revision with an edit receipt.
- Address a syntax occurrence within its document and revision. Shared green
  nodes, text offsets, and storage positions are not durable domain identities.
  Handles must reject another document or revision unless explicitly remapped.
- Lower a revision into validated typed data, or return diagnostics tied to
  that revision. Encode and save authored text through the format backend.

A revision carries a monotonically increasing per-document sequence number
for expected-base checks and a hash of its exact text. Equal text can occur at
different revision numbers; content equality does not revive stale handles.
Specify the content hash algorithm separately from the domain hash. A hash of
saved bytes helps detect external changes but does not make saving atomic.

**Save publication.** Save a named immutable revision and record which revision
actually reached disk. If revision 12 finishes saving while revision 13 is
current, the saved marker identifies 12 and the document remains dirty when
13 has different text. Serialize saves per document or otherwise prevent an
older save from overwriting a newer one. The backend must define external-change
detection, conflict handling, and file replacement before consumer adoption.
Comparing a disk hash and then writing has a race; atomic replacement alone
does not provide compare-and-swap against another writer. State the supported
coordination guarantees and test the chosen policy, including failed writes.

An occurrence handle is the document identity, the revision, and a Rowan
`SyntaxNodePtr` (node kind and text range). The first slice only rejects a
handle from another document or revision; remapping waits for a consumer that
needs it. `SyntaxNodePtr` locates nodes; token targets and insertion positions
may use revision-fenced ranges or format-specific locators.

A receipt should identify the document, base and resulting revisions, and
affected text ranges and optional semantic paths. Receipts always include
text ranges. A backend may recover paths from valid regions even when whole
document lowering fails; it must not imply those paths are validated domain
identities. TOML's logical structure is not its syntactic structure: in the
demo pack, `[[generators.lock_presets]]` is a
sibling header that belongs to the preceding `[[generators]]` entry, and dotted
keys may define one table in several places. A path such as
`generators[0].lock_presets[0]` can cover several disjoint syntax regions.
Document undo publishes a new revision containing the selected older content,
with a fresh sequence number and receipt. Earlier handles remain stale even
when the text matches again. Undoing an admitted application command remains
application-owned.

**Unknown content and identity.** The proof preserves and hashes unknown
properties that do not prevent typed decoding/validation, including the whole
`[vendor.experimental]` table. Changing opaque extension data changes the
domain hash; comments and formatting do not. Its text-only `GenValue` decoder
rejects unknown fields within that variant. This is a bounded proof policy,
not a product-wide ruling on unknown content or canonical domain encoding.

The proof's history limit and save/reload policy are recorded below. Product
retention budgets, external-writer coordination, crash recovery, and durable
identity/history remain adoption decisions. Shared API design follows evidence
from a second consumer.

## Pieces

| Piece | Intended role |
|---|---|
| Rowan 0.17 | Lossless authored syntax and structurally shared revisions. A format parser must build the tree, including unknown and erroneous input. |
| TOML parser | taplo 0.14 (MIT) builds lossless trees and collects syntax errors; DOM validation checks semantics such as duplicate keys. Its Rowan ^0.15.3 dependency resolved to 0.15.19 in the comparison. A manifest-only port to 0.17 passed the same tests and is used in the isolated document proof; the batched fork now also serves the private Isocosm CLI. Neither parser supports the tested TOML 1.1 additions; product typed decoding remains authoritative. |
| `toml_edit` | Existing TOML editing with comment and formatting preservation, for documents that parse. Existing uses remain useful; the private Isocosm CLI uses the batched Rowan backend. |
| Sonic | JSON ingestion and typed deserialization of machine interchange; not part of the authored-syntax slice. `OwnedLazyValue` remains an unmeasured candidate for deferred materialization. |
| Source overlay | The probe's byte-exact JSON overlay remains the candidate where machine JSON must survive byte for byte. Rowan does not replace it. |
| Serde and application schemas | Typed lowering, validation, and canonical domain encoding. |

The completed [October 13 probe](/Users/markik/Code/testing/redox-document-20261013/REPORT.md) measured Sonic, `serde_json`,
`toml_edit`, and the source overlay on the real 334-record monster catalogue
and the demo pack. Its overlay preserved untouched bytes exactly, `toml_edit`
preserved unrelated lines and comments, and the
[fresh-process reopening](/Users/markik/Code/testing/redox-document-20261013/results/fresh-reopen.json) passed. Those results
support the resource concept; they are not Rowan implementation or
performance evidence.

**Non-goal:** custom allocation and collection. Bump allocation and tracing
collection do not supply revision isolation, and `gc-arena` has its own
collection discipline. Keep both outside the initial document slice.

`cstree` 0.14 remains the comparison if shared, already-built navigation
across workers becomes necessary: its `SyntaxNode` is `Send + Sync`, at the
cost of retaining realized nodes. Rebuilding Rowan's navigation tree per worker
covers the expected cases. [cstree tradeoffs](https://docs.rs/cstree/0.14.0/cstree/).

The existing [format policy](2026-10-06_data_formats_brief.md)
continues to govern saved data: TOML for authored material, JSON for machine
interchange, framed postcard internally, and deterministic CBOR at the
specified boundaries. Domain hashes use the validated parsed form or canonical
encoding. Reformatting an authored document must not change domain identity.

## Bounded evidence and next gates

**Parser evaluation — 2026-10-15.** The
[comparison report](/Users/markik/Code/testing/retained-parser-20261015/REPORT.md)
and [results](/Users/markik/Code/testing/retained-parser-20261015/results/summary.json)
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

**Retained-document proof — 2026-10-16.** The isolated
[report](/Users/markik/Code/testing/retained-document-20261016/REPORT.md),
[syntax/save policy](/Users/markik/Code/testing/retained-document-20261016/SYNTAX.md),
and [machine-readable results](/Users/markik/Code/testing/retained-document-20261016/results/summary.json)
record an implemented resource and bounded pack consumer. The real
[demo pack](/Users/markik/Code/testing/retained-document-20261016/fixtures/demo-pack.toml)
and its synthetic
[annotated variant](/Users/markik/Code/testing/retained-document-20261016/fixtures/annotated-pack.toml)
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

**Wide-table profile and bounded optimization — experiments 2026-10-16;
recorded here 2026-10-17.** The
[stage profile](/Users/markik/Code/testing/retained-lowering-profile-20261016/REPORT.md)
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
[batching report](/Users/markik/Code/testing/retained-dom-batch-20261016/REPORT.md),
[paired results](/Users/markik/Code/testing/retained-dom-batch-20261016/results/summary.json),
and [patch](/Users/markik/Code/testing/retained-dom-batch-20261016/results/optimization.patch)
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

**Private Isocosm integration — published 2026-10-17.**
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

**Private CLI workflow qualification — 2026-10-17.** The
[workflow report](/Users/markik/Code/testing/pack-author-workflow-20261017/REPORT.md),
[reproduction receipt](/Users/markik/Code/testing/pack-author-workflow-20261017/reproduction/qualification.json)
and [command/response transcript](/Users/markik/Code/testing/pack-author-workflow-20261017/reproduction/transcript.json)
record two successful runs against Isocosm
[`285821ad`](https://github.com/merely-made/isocosm/commit/285821ad6663e75a289ea047769aa9dfb226195c),
after fetching and fast-forwarding the clean checkout. Each run uses 45 JSON-line
commands across five fresh CLI processes on a scratch copy of the real demo
pack with its existing annotated manifest. The scripted caller uses public CLI
responses only: an inspected value range locates the generator-name edit;
revision-checked text edits insert a replacement preset and remove the old one.
Real typed validation confirms ownership under the intended generator and no
preset leakage. Comments and the complete foreign-extension tail survive.
The raw transcripts contain five ready/exit pairs per run; the local report
and receipt incorrectly label that count as six. The 45-command count and
recorded pass/refusal outcomes agree in both runs.

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
new performance/memory behavior or crash safety. The report, runner, transcript,
scratch packs and exact intermediate/final manifests stay in local testing;
none are copied into Mere. Shared API extraction remains deferred.

**Next gates.** Keep this consumer private. A second consumer must establish
common lifecycle needs before extracting a shared API; shared API work remains
explicitly deferred. Durable identity/history, product retention and memory
budgets, wider platform/consumer compatibility, external-writer coordination
and crash behavior remain open. The private consumer resolves the real-schema
and existing-format compatibility question within its tested scope. Direct
structural edits, if introduced, must match parsing of their resulting text
before publication; that optimization path remains unexercised.

The detailed reports, fixtures, code, lockfiles and reproduction runners stay
in local testing directories: the
[October 13 probe](/Users/markik/Code/testing/redox-document-20261013/REPORT.md),
[October 15 parser comparison](/Users/markik/Code/testing/retained-parser-20261015/REPORT.md),
[October 16 document proof](/Users/markik/Code/testing/retained-document-20261016/REPORT.md),
[lowering profile](/Users/markik/Code/testing/retained-lowering-profile-20261016/REPORT.md),
[batched DOM experiment](/Users/markik/Code/testing/retained-dom-batch-20261016/REPORT.md),
and [private CLI workflow](/Users/markik/Code/testing/pack-author-workflow-20261017/REPORT.md).
Their absolute links resolve on the M4 host; the artifacts are not copied into
this repository or available through those links on GitHub.

## Sources checked 2026-10-14

Published crate sources from crates.io: rowan 0.16.1, 0.17.0 and
0.18.0-alpha.1; taplo 0.14.0; cstree 0.14.0; `toml_edit` 0.25.17.
At the October 14 check, Mere's
`origin/main` lockfile carried `toml_edit` 0.22.27 and 0.25.15 and did not
depend on Rowan, taplo, or cstree.
