# Retained document resource brief

**Date:** 2026-10-14. **Revised:** 2026-10-15. **Status:** Rowan is the
agreed working direction for authored syntax; the resource and its first consumer remain to be implemented.

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
Rowan consumer.

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
analysis of a fixed revision. The resource must still define history limits
and revision publication.
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
boundary. Define multi-edit ordering and overlap rejection before accepting
edit batches.

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

**Version choice.** Use a pinned 0.17 baseline for direct Rowan experiments;
evaluate taplo on its existing Rowan version as a private backend before
committing to a port. Rowan 0.18.0-alpha.1 (2026-09-14) attaches leading
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

**Unknown content and identity.** The annotated fixture carries a foreign
`[vendor.experimental]` table. Preserving it in text is settled. Its effect on
domain identity is not: lowering may exclude it from the hash, reject it at
admission, or carry it as an opaque extension that is hashed. Choose one
before the first consumer admits data.

History retention limits and the concrete save/reload conflict policy still
need decisions before consumer adoption; shared API design follows the second
consumer.

## Pieces

| Piece | Intended role |
|---|---|
| Rowan 0.17 | Lossless authored syntax and structurally shared revisions. A format parser must build the tree, including unknown and erroneous input. |
| TOML parser | taplo 0.14 (MIT) already parses TOML into a Rowan tree, collects syntax errors with offsets, and leaves semantic checks such as duplicate keys to its DOM. It depends on Rowan 0.15.3, whose types are incompatible with 0.17, and was last released in May 2025. Evaluate the unchanged crate as a private backend against a bounded port to 0.17. Check the selected TOML specification, malformed-input preservation, dependency compatibility, and maintenance cost before choosing. |
| `toml_edit` | Existing TOML editing with comment and formatting preservation, for documents that parse. Keep it as the backend until the Rowan consumer is proven. |
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

## First proof

The first proof should use one real pack/config consumer and an evaluated
lossless TOML parser, with the existing [demo pack](/Users/markik/Code/testing/redox-document-20261013/fixtures/demo-pack.toml) and its
[annotated variant](/Users/markik/Code/testing/redox-document-20261013/fixtures/annotated-pack.toml) as initial fixtures. It
must demonstrate:

- replacement, insertion and removal, including an entry under
  `[[generators.lock_presets]]`;
- preserved comments, the foreign extension table, and the literal string;
- an incomplete text edit that yields a retained revision with diagnostics;
- an unchanged older snapshot after later edits;
- stale-edit, wrong-document, and older same-document handle rejection;
- undo publishing a fresh revision without reviving older handles;
- UTF-8 byte-range validation, non-ASCII content, CRLF preservation, and
  explicit rejection of unsupported encodings;
- tree text equal to authoritative source and diagnostics tied to that revision;
- any optimized edit path equivalent to a fresh parse of the resulting text;
- correct typed lowering, and the chosen treatment of unknown content in the
  domain hash;
- save/reopen equivalence, and identical domain identity after reformatting;
- the selected external-change conflict behavior, failed-save behavior, and
  correct saved-revision tracking when a save overlaps a newer edit.

Measure full reparsing, edit application, cross-revision sharing, and retained
memory separately over repeated revisions, including any `NodeCache`. Include
a large flat TOML fixture with edits near its beginning, middle, and end to
expose wide-parent costs. Confirm that releasing history and resetting caches
releases allocations no longer reachable from snapshots or caches; process RSS
need not fall immediately. Factor a shared API once a second consumer
establishes which lifecycle operations genuinely belong in common.

The [probe report](/Users/markik/Code/testing/redox-document-20261013/REPORT.md) and [machine-readable results](/Users/markik/Code/testing/redox-document-20261013/results/summary.json)
retain the detailed measurements, fixture provenance, and reproduction steps.

The probe is an untracked local directory, `~/Code/testing/redox-document-20261013`,
on the M4 host; the links above are absolute paths into it.

## Sources checked 2026-10-14

Published crate sources from crates.io: rowan 0.16.1, 0.17.0 and
0.18.0-alpha.1; taplo 0.14.0; cstree 0.14.0; `toml_edit` 0.25.17.
At the October 14 check, Mere's
`origin/main` lockfile carried `toml_edit` 0.22.27 and 0.25.15 and did not
depend on Rowan, taplo, or cstree.
