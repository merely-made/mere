> Copied 2026-10-19 from the M4-local `~/Code/testing/source-edit-dedup-20261018/knot/REPORT.md`; see the [receipts index](../../README.md). Absolute `/Users/markik/...` paths below are host-local reproduction paths, not links.

# Knot immutable source snapshot deduplication

Date: 2026-10-18. Repository HEAD: 7e196c534e64b50cf6f00bf4d95e3beb493e5a9f.
Baseline: prior uncommitted shared source-edit Knot consumer slice, preserved
in baseline/ and baseline.json here. No commits/pushes or other-repo edits.

## Change

LinkForm now retains base: SourceSnapshot instead of address:String,
text:String and stamp:SourceStamp. Three fields become one (net -2). Three
local source identity/revision/byte comparisons delegate to SourceSnapshot.check.
DocKey, DocIdentity, source path, write posture, format, permission/readability,
selection, IME and target authority checks remain host-owned.

KnotDocumentSession provides source_snapshot(), checked selected_source(base,
selection), and propose_selection_edit(base,selection,text). The unpublished old
propose method signature was replaced, not retained as a shim. Proposal creation
uses base.propose so the immutable captured source/stamp remains the actual
proposal base. Common host selection validation and normalization are factored
once in the adapter and reused for preparation/application. The single input,
normal Edit refusal semantics, one undo unit, render/event source-version guard,
explicit Save and external-file baseline guard remain unchanged.

Added tests prove original base retention without mutation, stale refusal after
edit/Undo returns to identical bytes, and exact-source refusal when a caller
constructs a snapshot with the current stamp but different bytes. That forged
snapshot also refuses through the real link form without source or stamp change.

## Precise incremental size

Measured against baseline/, not repository HEAD:

- link_workflow.rs: +31/-18 lines overall; production before test module 492->482
  (-10), tests +23 net.
- source_edit.rs: +153/-27 overall; production 110->145 (+35), tests +91 net.
- knot-document/lib.rs: one import/reexport line replaced, no net lines.
- Production net +25; tests/adapted-test net +114; total Rust net +139.

This removes retained source state/guard duplication; it does not claim fewer
Rust lines overall. Shared supplier implementation changes are parent-owned and
are not included in these Knot-only counts.

## Final qualification

All positive Cargo commands below use --locked --offline and only this patch:

    --config 'patch."https://github.com/merely-made/mere.git".mere-source-edit.path="/Users/markik/Code/repos/mere/crates/system/source-edit"'

- document-tests.log: cargo test --manifest-path crates/knot-document/Cargo.toml
  --lib: 53 passed, one existing ignored.
- document-msrv-tests.log: same suite via cargo +1.97.1: 53 passed, one ignored.
- document-engine-tests.log: same with --features engine: 66 passed, one ignored.
- link-tests.log: cargo test -p knot-desktop --lib link_workflow::tests:
  11 passed (includes form-level forged snapshot and original ABA checks).
- link-ui-tests.log: cargo test -p knot-desktop --test document_links_ui
  -- --test-threads=1 --nocapture: four parent cases passed, one child helper
  ignored in the parent; each of TWO independent child processes runs its helper
  successfully. The real local two-file workflow keeps Unicode source/filenames,
  read-only target, selection, named relationship, explicit Save, one-step
  Undo/Redo, and exact reopening. A separate conflict path retains the draft,
  refuses overwrite and independently reopens the external author's bytes.
- workspace-link-ui-test.log: cargo test -p knot-desktop --lib
  document_link_menu_keeps_selection_and_cancel_escape_restore_source_focus:
  one passed. Opening, relayout, choosing a target, relayout and insertion work.
- document-lib-clippy.log: cargo clippy --manifest-path
  crates/knot-document/Cargo.toml --lib -- -D warnings: passed.
- desktop-clippy.log: cargo clippy -p knot-desktop --lib --test document_links_ui:
  passed with existing workspace warnings; no warning names either changed
  source file. The prior strict test-target failures in unchanged hard_wrap.rs
  (useless_format) and writer.rs (items_after_test_module) were not cleaned up.
- rustfmt --edition 2024 --check crates/knot-document/src/lib.rs
  apps/desktop/src/link_workflow.rs apps/desktop/tests/document_links_ui.rs and
  git diff --check pass.

Source/model and UI-harness plus independent fresh-process qualification only;
no native GUI observation or whole-product claim.

## Isolation/publication

Incrementally changed only:

- apps/desktop/src/link_workflow.rs
- crates/knot-document/src/lib.rs
- crates/knot-document/src/source_edit.rs
- design_docs/2026-09-29_knot_document_links_slice.md
- design_docs/README.md

Existing prior changes in Cargo.lock, Cargo.toml, document_surface.rs,
document_view.rs and the desktop UI test are byte-identical to this baseline.
The ignored standalone Cargo.lock is also unchanged. No supplier/package repin
or manifest change is part of this follow-on. Prior testing receipts were not
modified. qualification.json contains exact baseline/current hashes and counts.

Normal Git-only qualification remains outstanding: the shared crate is
unpublished, so the consumer's canonical Git main dependency still needs the
explicit local patch for qualification. The shared API itself is authorized
and used; it is not deferred. Supplier publication, clean Git-resolution
qualification and any commits/pushes are separate actions.
