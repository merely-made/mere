> Copied 2026-10-19 from the M4-local `~/Code/testing/source-edit-dedup-20261018/REPORT.md`; see the [receipts index](../README.md).

# Source-edit duplication removal

Status: qualified locally, uncommitted and unpushed. This continues the prior
uncommitted shared-source-edit slice rather than replacing its evidence.
Fetched all three repositories first. Mere was fast-forwarded to `dd9d5c523`
with only its own index edits temporarily preserved and reapplied; Isocosm
remains `183a2f67`, Knot `7e196c534`. Upstream changes and prior work remain.

## What was removed

Knot's `LinkForm` no longer independently retains `address`, `text`, and `stamp`.
One shared immutable `SourceSnapshot` captures that observation, checks it and
constructs proposals from it. Three fields become one (two net fields removed),
and the form's handwritten source identity/revision/byte comparisons move to
the shared guard. Proposals use the original captured base rather than silently
recapturing newer text when submitted.

Checked selection reading, range normalization and proposal preparation are
centralized in Knot's session adapter. DocKey/DocIdentity, source path, format,
permissions/readability, targets, selection and IME remain application checks.
The existing editor, message-driven revision tracking, normal Edit refusal
transitions, one-step undo, explicit Save and file baseline remain unchanged.

Isocosm edit and undo use one checked edit/application/publication path. Undo
is an ordinary full-source replacement. The app no longer hand-builds undo's
four range coordinates; publication takes the checked revision and ranges from
the shared application instead of performing a second revision increment.
Format preflight and snapshot construction still finish before current/history
publication. Schema, semantic confidence, wire receipts, saving and admission
remain application-owned.

The shared snapshot owns immutable base bytes through an Arc; clones and
proposals share that base. It is not another mutable editor or storage owner.
Range computation and all source-stamp/byte guards have one shared implementation.

## Measured structural change

Against the captured previous working-tree slice, Knot's production link
workflow shrinks ten lines. Reusable session helpers add 35 production lines,
so overall Knot production grows by 25 lines; tests grow by 114. This removes
duplicated state/checks, not total LOC. [Knot report](knot/REPORT.md) and its
[receipt](knot/qualification.json) retain exact baseline files and per-file
increments. No safety check was deleted just to reduce line count.

## Verification

| Owner | Checks |
|---|---|
| Core | Six focused tests; strict all-target Clippy; tests on Rust 1.97.1. |
| Isocosm | 75 feature/58 default tests; scoped all-target Clippy; Rust 1.97.1 binary check; binary/example build; 59 commands across five processes. |
| Knot | 53 default/66 engine document tests, one existing ignore each; default suite on Rust 1.97.1; 11 link-model cases; four file-workflow harness cases and two actual child-process reopens; one menu/focus scenario; strict library and scoped desktop Clippy. |

The core test proves one captured source remains the base for multiple proposals,
and stale revision, wrong instance, and changed bytes paired with the current
stamp refuse. Existing Unicode, batch, overlap and receipt tests still pass.
[Core receipt](core/qualification.json), [exact checks](core/checks.json).

Isocosm controls verify the exact undo receipt, stale and no-op undo, and both
edit/undo overflow refusal with no history/current/save mutation. All 64
ready/command records in the real workflow match the prior shared adapter after
normalizing only scratch paths. Invalid draft repair, explicit saving, external
conflict recovery and actual loader/fixture execution pass.
[Receipt](isocosm/qualification.json), [transcript comparison](isocosm/protocol-comparison.json),
[workflow](isocosm/workflow/qualification.json).

Knot's new tests reject forged captured bytes paired with a current stamp through
both the session and real link form, and reject stale source after an equal-text
edit/Undo cycle. Refusal leaves source and revision unchanged. Prior UI-harness
checks cover layout stability, source/target authority, Unicode, read-only target,
directional selection, IME, one undo unit, explicit saving, external-file refusal
and fresh-process reopens. No reviewed native GUI observation is claimed.
Existing strict all-test lint failures in unchanged files are not a passing gate.

## Dependency and publication boundary

All positive consumer checks use the recorded explicit local Cargo override for
only the new `mere-source-edit` package. Existing manifests and locks are unchanged
by this cleanup; no other supplier is repinned. Normal Git-only resolution is
still unqualified until the supplier is published and remote consumer locks are
generated. This pass performs no commits/pushes or global Cargo configuration.
Earlier qualification artifacts remain unchanged; new logs and receipts are here.

No Woodshed changes, broad mutable-input rewrite, parser replacement, automatic
merge, crash-recovery or platform-wide acceptance is included. The scopes and
remaining publication boundary are also recorded in the owning plans/indexes.
