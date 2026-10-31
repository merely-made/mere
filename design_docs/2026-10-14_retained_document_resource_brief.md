# Retained document resource brief

**Date:** 2026-10-14. **Revised:** 2026-10-20.

**Status:**

- Rowan 0.17 is the working direction for retained authored documents.
- Isocosm has a private, optional `pack-author` CLI on the
  [merely-made/taplo](https://github.com/merely-made/taplo) fork (Rowan 0.17,
  batched DOM, opt-in TOML 1.1), with text-only history by default,
  PID-checked lease reclaim and structure-aware commands.
  It uses the real pack schema and passes a real-pack workflow.
- The shared [`mere-source-edit`](archive_docs/2026-10-28/2026-10-18_source_edit_proposal_plan.md)
  proposal crate is published, and Isocosm and Knot both consume it.
- Mark ruled on history, stale leases, TOML 1.1, the fork's home and CLI
  commands on 2026-10-19 ([decisions](#decisions-2026-10-19)).
- Durable history, memory budgets and crash behavior remain open.
- Dated evidence is in the [evidence log](mere_docs/testing/receipts/2026-10-19_retained_document/evidence_log.md),
  with copied [receipts](mere_docs/testing/receipts/2026-10-19_retained_document/README.md).

Use a retained document resource to inspect and revise authored data while
preserving its text, comments, unknown content, and earlier revisions. The
resource owns document identity, revision history, diagnostics, and edit
receipts. Rowan supplies the syntax tree. Format parsers and typed views
remain specific to the application and format.

**Why Rowan for TOML when `toml_edit` already preserves comments.**
`toml_edit` does not expose a retained syntax tree for a document that fails
parsing: `Document::parse` and `FromStr` for `DocumentMut` return a document
or a `TomlError`. A resource must still retain that source, its parse error,
and earlier revisions, and syntax-aware editing of incomplete input needs an
error-tolerant parser. Rowan supplies the immutable tree; the TOML parser must
preserve all input and produce diagnostics. `toml_edit` stays useful for
documents that parse.

## Rowan revisions

Rowan's immutable green tree lets revisions share unchanged subtrees. Rowan
0.17.0 is immutable only: 0.16's in-place mutation API (`clone_for_update`,
`splice_children`, `detach` on syntax nodes) was removed, so a retained root
cannot change under a reader. `SyntaxNode::replace_with` asserts the
replacement has the same kind and returns a new green root, rebuilding
ancestors to the root while sharing unchanged subtrees. Each rebuilt parent
processes its child list, so edit cost depends on ancestor widths as well as
depth. Insertion and removal go through the parent's green node
(`insert_child`, `remove_child`, `splice_children`) and then `replace_with`.
Retaining the old root preserves the previous revision, which supports
previews, document undo, and analysis of a fixed revision.
[Rowan replacement API](https://docs.rs/rowan/0.17.0/rowan/api/struct.SyntaxNode.html#method.replace_with).

**Interning and memory.** Only the green-node builder interns through
`NodeCache`; structural edits allocate fresh nodes outside it. `NodeCache` is
a plain hash map with no eviction API, so a long-lived cache shared across
reparses keeps its entries until dropped, and dropping history alone cannot
release them. Retained-memory measurements must include the cache as well as
retained roots. Full reparsing with a fresh cache shares no nodes between
revisions (measured 2026-10-15).

**Workers.** Green nodes are `Arc`-backed and `Send + Sync`. Red
`SyntaxNode`s use a non-atomic reference count and are neither. A worker
receives a green root and builds its own navigation tree cheaply, so no second
library is needed for worker ownership. `cstree` 0.14 remains the comparison
if shared, already-built navigation across workers becomes necessary: its
`SyntaxNode` is `Send + Sync`, at the cost of retaining realized nodes
([tradeoffs](https://docs.rs/cstree/0.14.0/cstree/)).

**Version choice.** taplo 0.14 moved to Rowan 0.17 with manifest changes only
and zero Rust source changes; the port passed the same tests (2026-10-15).
Rowan 0.18.0-alpha.1 attaches leading and trailing trivia to tokens, so
comments and whitespace move into the token they decorate and `index()`
becomes optional. That changes where a parser places comments and what an
edit carries with it; revisit comment ownership at 0.18.

## Resource contract

- Open and retain a document with an explicit format and document identity.
- Read or snapshot a named revision without changing it.
- Apply an edit against an expected base revision; reject stale bases and
  publish a new revision with an edit receipt.
- Address a syntax occurrence within its document and revision. Shared green
  nodes, text offsets, and storage positions are not durable domain identities.
  Handles must reject another document or revision unless explicitly remapped.
- Lower a revision into validated typed data, or return diagnostics tied to
  that revision. Encode and save authored text through the format backend.

An occurrence handle is the document identity, the revision, and a Rowan
`SyntaxNodePtr` (node kind and text range). The first slice only rejects a
handle from another document or revision; remapping waits for a consumer that
needs it. Token targets and insertion positions may use revision-fenced ranges
or format-specific locators.

A receipt identifies the document, base and resulting revisions, affected text
ranges, and optional semantic paths. Receipts always include text ranges; a
backend may recover paths from valid regions even when whole-document lowering
fails, but must not imply those paths are validated domain identities. TOML's
logical structure is not its syntactic structure: `[[generators.lock_presets]]`
is a sibling header that belongs to the preceding `[[generators]]` entry, and
dotted keys may define one table in several places, so a path such as
`generators[0].lock_presets[0]` can cover several disjoint syntax regions.

## Editing and admission

A document revision may contain incomplete or invalid text; its parser
preserves that text and attaches diagnostics. Saving a draft preserves
authored bytes. Lowering a revision into typed application data requires
schema and semantic validation, and the owning application decides whether to
admit the result. A syntax resource gains no authority over world state,
grants, or replication. Document undo after admission reverts text, not
admitted state; the application must make that visible.

**Unknown content and identity.** The proofs preserve and hash unknown
properties that do not prevent typed decoding, such as a whole
`[vendor.experimental]` table. Changing opaque extension data changes the
domain hash; comments and formatting do not. This is a bounded proof policy,
not a product-wide ruling on unknown content or canonical domain encoding.
The [format policy](2026-10-06_data_formats_brief.md) governs saved data:
TOML for authored material, JSON for machine interchange, framed postcard
internally, deterministic CBOR at the specified boundaries. Domain hashes use
the validated parsed form or canonical encoding; reformatting an authored
document must not change domain identity.

## Revision and save invariants

- **One text per revision.** Every published revision has one authoritative
  authored text; its syntax tree and diagnostics describe exactly that text.
  Direct structural edits must match parsing of the resulting text, or reparse
  before publication. Green-tree splicing alone does not establish this:
  token boundaries, separators, and error recovery can change.
- **Edit path.** Start with text edits and a full reparse. Syntax-aware
  commands compute text edits from the current tree and publish through the
  same path. Incremental reparsing and direct splicing are later candidates if
  measurements justify them and equivalence checks pass. A receipt names both
  coordinate spaces: base-revision ranges replaced and result-revision ranges
  produced.
- **Bytes and coordinates.** Accept UTF-8 and preserve bytes, including line
  endings, without normalization; reject other encodings explicitly. Edit and
  diagnostic ranges are UTF-8 byte offsets on code point boundaries. Batch
  edits sort in base coordinates; overlaps and same-start edits, including
  ambiguous same-position insertions, are refused.
- **Sequence and hash.** A revision carries a monotonically increasing
  per-document sequence number for expected-base checks and a hash of its
  exact text. Equal text at a later revision does not revive stale handles.
  Undo publishes the older content as a fresh revision with its own receipt.
  The content hash algorithm is specified separately from the domain hash.
- **Save publication.** Save a named immutable revision and record which
  revision reached disk. If revision 12 finishes saving while 13 is current,
  the saved marker says 12 and the document stays dirty if 13's text differs.
  Serialize saves per document so an older save cannot overwrite a newer one.
- **External writers.** A saved-bytes hash detects external changes but does
  not make saving atomic. Comparing a disk hash and then renaming has a race,
  and atomic replacement alone is not compare-and-swap against another writer.
  The backend states its coordination guarantees and tests them, including
  failed writes. The current backend serializes cooperating writers with a
  per-path lease, checks disk hashes and renames a synced same-directory
  temporary file; conflicts require explicit reopen.

## Pieces

| Piece | Intended role |
|---|---|
| Rowan 0.17 | Lossless authored syntax and structurally shared revisions. A format parser must build the tree, including unknown and erroneous input. |
| TOML parser | The taplo 0.14 (MIT) fork at [merely-made/taplo](https://github.com/merely-made/taplo), branch `merely/rowan-0.17`: Rowan 0.17, batched DOM construction, `parse_with_cache`, and opt-in TOML 1.1 (TOML 1.0 stays its default). Isocosm pins it by exact revision. It builds lossless trees and collects syntax errors; DOM validation checks semantics such as duplicate keys. Product typed decoding remains authoritative. |
| `toml_edit` | Existing comment- and format-preserving editing for documents that parse. Existing uses remain. |
| `mere-source-edit` | Parser-neutral UTF-8 edit proposals with instance/revision/exact-source checks, previews and candidate receipts. Hosts keep their buffers, undo, save and admission. See the [source-edit plan](archive_docs/2026-10-28/2026-10-18_source_edit_proposal_plan.md). |
| Sonic | JSON ingestion and typed deserialization of machine interchange; not part of the authored-syntax slice. `OwnedLazyValue` remains an unmeasured candidate for deferred materialization. |
| Source overlay | The byte-exact JSON overlay from the 2026-10-13 probe remains the candidate where machine JSON must survive byte for byte. Rowan does not replace it. |
| Serde and application schemas | Typed lowering, validation, and canonical domain encoding. |

**Non-goal:** custom allocation and collection. Bump allocation and tracing
collection do not supply revision isolation, and `gc-arena` has its own
collection discipline. Both stay outside the document slice.

## Decisions 2026-10-19

Mark's rulings from the 2026-10-19 review:

1. **History.** Try two strategies and measure them against each other:
   text-only history, which keeps a syntax tree only for the current revision
   and pinned snapshots, and a document-scoped `NodeCache` shared across
   reparses. Add a byte budget alongside the existing 32-revision cap.
2. **Stale save lease.** Reclaim a stale lease by checking the PID recorded
   in it.
3. **TOML 1.1.** Packs get TOML 1.1 by teaching the taplo fork 1.1. The fork
   moves from Isocosm's `vendor/` into its own merely-made repository.
4. **Pack-author CLI.** The CLI gains structure-aware commands that emit
   `mere-source-edit` proposals. Callers no longer have to work out offsets
   and TOML ownership themselves.

**Outcome, 2026-10-20.** All four landed in Isocosm
[`2389818c`](https://github.com/merely-made/isocosm/commit/2389818c). Text-only
history is the default: at 65 revisions it retains 98,944 bytes against 849,544
for full trees and 537,240 for the shared `NodeCache`, and publishes within
noise of full. The cache mode shares 90 of 152 green nodes and remains
available behind `--history`. On 2026-10-20, leases also reclaim reused PIDs
and earlier boots, the fork lays out TOML 1.1 multiline inline tables, and
Isocosm resolves on cargo 1.97.1. Details are in the
[evidence log](mere_docs/testing/receipts/2026-10-19_retained_document/evidence_log.md#isocosm-continuation--2026-10-19).

## Open decisions

- Durable document identity and history across processes.
- Product-wide policy for unknown content and canonical domain encoding.
- Which history strategy to keep, and the byte budget's value, once both
  strategies are measured.
- Crash recovery beyond stale-lease reclaim, directory fsync, and behavior
  with noncooperating writers, path aliases and nonlocal filesystems.
- Handle remapping across revisions, when a consumer needs it.
- Comment ownership under Rowan 0.18's token trivia.

## Next gates

- Upstreaming the fork means rebasing onto upstream's newer master.
  Packaging a tool distribution requires qualifying the fork's packaging.
- Two `isometry-genet` `host_zoom` layout-figure tests fail on Isocosm main;
  they predate the 2026-10-20 repin.
- Wider platform and consumer compatibility, peak memory/RSS, and native GUI
  acceptance remain unqualified.
- Direct structural edits, if introduced, must match parsing of their
  resulting text before publication; that path is unexercised.

## Sources checked 2026-10-19

Published crate versions on crates.io: rowan 0.17.0 is the latest stable
(0.18.0-alpha.1 is still the only newer release); taplo 0.14.0; `toml_edit`
0.25.17; `toml` 1.1.8; cstree 0.14.0. Mere's `origin/main` lockfile at
`48fd33851` carries `toml_edit` 0.22.27 and 0.25.15, `toml` 0.8.23, 0.9.12 and
1.1.6, and `mere-source-edit` 0.1.0, and does not depend on Rowan, taplo or
cstree. The first check, on 2026-10-14, covered rowan 0.16.1, 0.17.0 and
0.18.0-alpha.1, taplo 0.14.0, cstree 0.14.0 and `toml_edit` 0.25.17.
