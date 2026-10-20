# Shared source-edit proposals

**Date:** 2026-10-18. **Status:** supplier and both adapters published with Git-only receiving qualification. Supplier:
[Mere `70deef2c8`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f).
Initial local-override evidence is retained in the
[implementation report](mere_docs/testing/receipts/2026-10-19_retained_document/shared-source-edit-20261018/REPORT.md)
and [deduplication report](mere_docs/testing/receipts/2026-10-19_retained_document/source-edit-dedup-20261018/REPORT.md).

## Context and decision

The retained-document probes and private Isocosm pack authoring established
revision-checked edits, previews and explicit saving. Inspection of Knot found
the same operation in an existing Djot local-link workflow: it captures the
document, text, selection and target, rechecks them, then applies a source edit.
The user authorized a shared API, two consumer adapters and real-workflow
qualification. Woodshed's arrangement drafts are a valid later candidate; its
existing typed state and sealed storage need no new source-buffer owner here.

Use a small parser-neutral `mere-source-edit` crate under `crates/system/`.
The common contract is a document-instance/revision stamp, validated UTF-8 text
edits, immutable preview and candidate application receipt. Retaining all of
Isocosm's document/session API as the shared interface would force unrelated
parser, history and saving choices on Knot. Leaving identical range/proposal
checks app-local would miss a concrete second consumer. This slice shares only
the demonstrated common operation.

## Ownership

- The shared crate checks ranges, batch ambiguity, exact source and expected
  document-instance/revision. It computes text and both coordinate spaces.
- Hosts assign fresh instances, advance revisions including undo/redo, serialize
  check/application, and retain their existing single editor/source authority.
- Knot retains selection/IME/undo, source-format and target authority, save and
  recovery policies. A proposal is applied as one undoable source operation.
- Isocosm retains its Rowan/TOML backend, layout confidence checks, real schema,
  syntax diagnostics, history, explicit save and application admission policy.
- No source stamp or receipt is a durable identity, authorization token, domain
  hash, persistence receipt, or claim that an application has occurred.

## Phases and done-conditions

- S1: shared proposal contract and focused tests. Done when malformed UTF-8 byte
  boundaries, overlaps/same-start edits, wrong instances, changed source, stale
  revisions, equal-text undo and revision exhaustion refuse; preview/application
  leave inputs unchanged and receipts describe both coordinate spaces.
- S2: Isocosm adapter. Done when the private preset workflow uses the shared
  edit contract, retains its current protocol and semantic/layout checks, and
  the real scratch-pack workflow passes with explicit edit/save separation.
- S3: Knot adapter and qualification. Done when existing local-link authoring
  uses the contract without a second editor buffer, preserves captured source
  and target authority/selection/IME checks, and proves one-step undo, stale and
  wrong-document refusal, saved-byte reopen and external-file refusal.
- S4: integration review. Done when supplier and consumer source identities,
  scoped checks and real workflow evidence are recorded separately from native
  GUI review and publication. No broader platform or conformance claim follows.

## Findings

2026-10-18: both consumer source paths are real. Knot's `LinkForm` already holds
source identity/path/address/text/selection and target identities, and checks
them immediately before the source edit. Its document session has source-bound
readouts and guarded disk saving. Isocosm's `Session::edit` already checks base
revision, boundaries and overlap before publishing a new immutable revision.
Neither path needs a common parser or replacement storage owner.

The proposed application returns a candidate result stamp and text, not an
applied receipt. Hosts publish them only after their own application succeeds.
An exact-source guard supplements, rather than replaces, monotonic revisions.
The shared object cannot observe host mutation or create authority.

## Qualification and publication boundaries

Workflows and raw receipts live in
`Code/testing/shared-source-edit-20261018/` on the M4 host (reports and small
receipts copied to [receipts](mere_docs/testing/receipts/2026-10-19_retained_document/README.md)), with `core/`, `isocosm/`, and `knot/`
owners. Consumer manifests use the canonical Mere Git URL. Initial local
qualification used `branch = "main"`; the publication continuation below records
the bounded Knot revision pin. Before the new package was published, explicit local
Cargo patch configuration may qualify only `mere-source-edit`; it must not
silently repin other Mere suppliers. The local-override basis, resulting lock
shape and normal remote-resolution limitation must be recorded.

The subsequent user request authorizes supplier publication followed by scoped
consumer lock updates, unpatched receiving checks and consumer publication.
Replacement of live file stores is outside this slice. Fresh-process reopening and automated host
tests do not by themselves constitute reviewed native GUI qualification.

## Progress

- 2026-10-18: fetched and fast-forwarded three clean checkouts, recorded
  baselines and split disjoint ownership between shared core, Knot adapter and
  Isocosm adapter. Shared contract is implemented; consumer checks are pending.

## Bounded results

The core passes five focused tests, strict all-target Clippy and its tests on
Rust 1.97.1. Independent review found no blocking correctness issue and led to
explicit all-or-nothing batch/undo and base-coordinate replay obligations.

Isocosm passes 74 feature-enabled tests, 58 default tests, all-feature/all-target
compilation, scoped Clippy and Rust 1.97.1 compilation with the explicit local
patch. Its real workflow passes 59 commands across five processes. All 64
ready/command records match the previous private CLI after normalizing only
scratch-directory paths. Parser/schema/save/admission policy remains app-local.

Knot passes 51 default document tests and 64 engine-enabled tests (one existing
ignore in each), the default tests on Rust 1.97.1, ten link-form model cases,
four two-file UI-harness cases and one existing workspace menu/focus case.
Two actual child processes reopen saved/external files; the helper is ignored
in the parent suite and explicitly executed in those children. Library Clippy
passes; strict all-test Clippy has existing unchanged-file failures recorded
in the consumer evidence. No native screenshot review is claimed.

The Knot adapter accepts single-selection proposals as one undoable edit;
multi-range applications refuse until that host has a batch undo boundary.
A transient before-message source witness advances revisions only for actual
textarea changes, not build/rebuild. Public raw mutable-input borrowing
conservatively invalidates proposals. Selection-only ordinary intents are
stamp-neutral; source edits, undo/redo, reload and redirected Save As invalidate
older proposals. Existing source/target checks, IME refusal and file guards stay.

All three workspace locks preserve existing package versions and source
revisions; only the new package is added. Consumer locks currently identify
that explicit local patch. Normal remote-only source resolution is not yet
qualified because the new package is absent from published Mere main. Supplier
publication, scoped lock generation and unpatched receiving checks remain the
next integration step; no commit/push has been performed.

The new plan has a D2 record. The repository-wide document audit still fails
on the unchanged HEAD snapshot digest and 18 pre-existing missing records;
`core/doc-audit-baseline.json` verifies those blockers predate this change.
Unrelated audit records were not rewritten.

- 2026-10-18: S1–S4 complete within the explicit-local-supplier qualification
  boundary. Core, Isocosm and Knot reports retain exact source/check identities;
  publication and native desktop observation remain separate.

## Duplication-removal continuation

The user authorized using the shared contract to remove app bookkeeping. Mere
was fetched and refreshed to `dd9d5c523`, preserving all uncommitted work and
upstream changes. This pass remains uncommitted and uses the explicit local
new-package override; remote publication is not inferred from local checks.

- S5 locally qualified: one shared immutable `SourceSnapshot` owns the captured stamp
  and source observation, its guard and proposal construction. Knot replaces
  form-owned address/text/stamp storage and handwritten equality checks with
  that observation. Path, target, selection, IME and authority checks stay local.
- Isocosm undo now constructs an ordinary replacement edit and uses the same
  application/publication path as edits. Publication takes the checked result
  revision and ranges, removing hand-built undo ranges and a second increment.
- Done when the previous real workflows remain equivalent, snapshot checks
  retain stale/instance/exact-source guards, undo receipts and overflow refusal
  remain transactional, and one-step editor undo and file guards still pass.

S5 results: six core tests/strict Clippy/Rust 1.97.1 pass. Isocosm passes 75
feature/58 default tests and an identical normalized 59-command workflow.
Knot passes 53 default/66 engine document tests (one existing ignore each),
11 link-model cases, four file-workflow cases and two child reopens, plus the
menu/focus scenario. The new test base is
`Code/testing/source-edit-dedup-20261018/`; earlier evidence stays unchanged.
Knot's three separate source-capture fields become one shared snapshot (two
net fields removed); three handwritten source comparisons are shared checks.
Its production workflow shrinks ten lines, but extracted adapter helpers add
35; this is state/guard consolidation, not an overall LOC reduction. Isocosm
removes hand-built undo ranges and a second revision computation.

## Published supplier and Git-only intake — 2026-10-18

The user authorized supplier-first publication, then unpatched receiving checks
and scoped consumer publication. Mere was fetched and the supplier commit
rebased onto `908283728`; the published implementation is
[`70deef2c8`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f).
Six tests, strict all-target Clippy and Rust 1.97.1 tests pass on that source.
The earlier local-override results above are historical qualification stages.

Isocosm's [consumer commit `276157b7`](https://github.com/merely-made/isocosm/commit/276157b776c59394923550b0b42d543b317210d9)
uses canonical Git main, with the committed lock resolving the new supplier at
`70deef2c8`. Its 75 feature/58 default tests, all-feature/all-target check,
scoped Clippy, Rust 1.97.1 binary check and build pass without overrides.
The real workflow passes 59 commands/five fresh processes and all 64 normalized
ready/command records match the preceding qualification. Default normal runtime
dependencies exclude the optional source-edit/Taplo/Rowan tooling.

Knot isolates only the new supplier with the canonical Git URL and exact
published `rev = "70deef2c8cf3d3410ae35b496a9396514070b24f"`. A common `main`
source would unify existing Mere dependencies at that revision, requiring a
broader supplier update and exposing unrelated desktop API changes. The explicit
remote revision avoids that update without a local override. Cargo generates
both root and standalone document locks; existing package versions and source
revisions remain unchanged. This is a deliberate bounded exception to the
usual first-party main-branch convention.

The [publication report](mere_docs/testing/receipts/2026-10-19_retained_document/source-edit-publication-20261018/REPORT.md)
retains fresh receiving logs, resolved Git metadata and workflow receipts.
Old testing evidence is unchanged. The repository-wide documentation audit has
exactly the same pre-existing diagnostics as the preceding pass, recorded in
`core/doc-audit-comparison.json`. GUI review and broader compatibility remain
separate claims.

Knot's [consumer commit `d2d5c32`](https://github.com/merely-made/knot-editor/commit/d2d5c32942d40af47223790ed3e16bc88fa59aa6)
passes 53 default/66 engine document tests, 53 tests on Rust 1.97.1 (one existing
ignore in each), 11 link-model cases, four desktop workflow cases, two fresh
child-process reopens and the menu/layout/focus scenario. Strict library Clippy,
scoped desktop Clippy, formatting and diff checks pass with baseline warnings
and the previous strict all-test lint limitation retained. Both consumer commits
are pushed to their respective origin main branches. Only this work was staged.
