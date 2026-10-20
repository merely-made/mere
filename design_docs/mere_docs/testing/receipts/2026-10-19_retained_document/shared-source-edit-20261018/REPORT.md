> Copied 2026-10-19 from the M4-local `~/Code/testing/shared-source-edit-20261018/REPORT.md`; see the [receipts index](../README.md).

# Shared source-edit proposal qualification

Status: core and two consumer adapters are locally qualified in working trees.
No files are staged, committed or pushed. Publication and remote-only receiving
checks remain separate. Baselines: Mere `e6250fd1c`, Isocosm `183a2f67`, Knot
`7e196c534`. All were fetched and fast-forwarded cleanly before implementation.
Woodshed was not modified.

## Implemented boundary

`mere-source-edit` prepares parser-neutral UTF-8 proposals bound to document,
open instance, revision and exact source. It validates/sorts base-coordinate
ranges, rejects overlap/same-start ambiguity, computes a preview, and checks
the host's current observation before returning candidate text and base/result
receipts. It never edits a buffer, parses a format, writes disk, records undo,
grants authority or admits application data.

The host checks and applies atomically at its existing editor boundary.
Replaying `edits()` requires descending offsets or simultaneous semantics.
The whole proposal must be one undoable transaction or refuse without source,
stamp or undo changes. A host without batch undo accepts only one range.
Stamps/candidate receipts are not durable identities or persistence claims.

## Results

| Owner | Observed checks |
|---|---|
| Core | Five focused tests; strict all-target Clippy; tests on Rust 1.97.1; independent bounded review with no blocking correctness finding. |
| Isocosm | 74 feature/58 default tests; all-feature/all-target check; scoped Clippy; Rust 1.97.1; 59 commands/five processes and real loader/fixture execution. |
| Knot | 51 default/64 engine document tests, one existing ignore each; default suite on Rust 1.97.1; ten link-model cases; four two-file UI-harness cases; two explicit child-process reopens; one workspace menu/focus case; strict library Clippy and scoped desktop Clippy. |

Core tests cover Unicode/CRLF, mixed edit sizes, both coordinate spaces,
malformed boundaries, overlaps, independent stamp/source guards, equal-text
undo and exhaustion. Independent review led to clearer transaction/replay
documentation. [Checks](core/checks.json), [receipt](core/qualification.json),
[review](core/REVIEW.md).

Isocosm's edit and preset-preview paths use the kernel; its parser, TOML
confidence/ownership checks, schema, publication, history, undo, save and
admission remain app-local. All 64 ready/command records match the previous
private workflow after normalizing only scratch-directory paths. Invalid-draft
repair, refusal controls, preservation, external-conflict recovery and actual
fixture execution pass. [Report](isocosm/REPORT.md),
[receipt](isocosm/qualification.json), [comparison](isocosm/protocol-comparison.json).

Knot keeps its existing Djot local-link workflow and single editor. It rechecks
source/target identity, path, selection, write posture and IME. One selection
replacement applies through normal edit semantics with one-step undo/redo and
explicit guarded saving; multi-range proposals refuse. A transient
before-message witness counts source changes without invalidating forms during
build/rebuild. Public mutable escape hatches invalidate conservatively. Tests
cover layout stability, stale/ABA, wrong instance, Save As redirection, focus,
selection, authority, save and independent-process reopening. External edits
refuse save while retaining the draft. [Report](knot/REPORT.md),
[receipt](knot/qualification.json).

The two child reopens are actual independent processes. The helper is ignored
in the parent suite and explicitly executed in each child. No native GUI
capture or whole-product acceptance is claimed. Knot strict test-target Clippy
still reports existing unchanged-file issues; passing scoped/library checks
are recorded separately.

## Source identity and publication

Manifests name canonical Mere Git `branch = "main"`. Positive consumer checks
use explicit local Cargo patches for **only** `mere-source-edit`, since it is
absent from published Mere main. All existing locked versions/source revisions
are retained; [lock audit](lock-audit.json) identifies the new package. Consumer
locks currently describe that local patch, not a portable remote package.

Normal unpatched resolution is not green. Publish the supplier when authorized,
generate scoped remote locks and rerun unpatched receiving checks before claiming
Git-source adoption. This pass performs no commits/pushes or persistent global
Cargo override. Exact commands and exit statuses are in owner check records.

The Mere plan/brief/index describe a shared proposal slice, not a complete
retained-document session or a requirement that Knot use Rowan. The new plan
has D2 coverage. The repository-wide audit retains an unchanged baseline
snapshot-digest mismatch and 18 pre-existing missing judgments; the
[baseline check](core/doc-audit-baseline.json) separates them from this change.
No unrelated audit cleanup was performed.

All raw reports, logs, scratch sources and receipts remain here in testing.
Publication, native observation and fresh-process source behavior are separate
claims; none is inferred solely from successful compilation.
