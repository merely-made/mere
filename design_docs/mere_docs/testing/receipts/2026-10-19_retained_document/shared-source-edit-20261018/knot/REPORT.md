> Copied 2026-10-19 from the M4-local `~/Code/testing/shared-source-edit-20261018/knot/REPORT.md`; see the [receipts index](../../README.md). Absolute `/Users/markik/...` paths below are host-local reproduction paths, not links.

# Knot shared source-edit consumer qualification

Date: 2026-10-18. Base: Knot 7e196c534e64b50cf6f00bf4d95e3beb493e5a9f.
Status: implemented and qualified in working tree; no commits or pushes.

The existing two-document link workflow uses the new shared Proposal contract
via an additive KnotDocumentSession adapter. One retained TextInput remains the
only mutable document buffer. The adapter accepts one replacement over the
captured selection. Shared source/stamp/UTF8 checks precede mutation; all existing
source authority, target identity/path/readability, selection, and IME guards
remain. Successful insertion uses the normal Edit intent and clears refusal
consistently; it breaks typing coalescence before/after to remain one undo step.
It does not save implicitly or change the existing external-file save guard.

The first draft's mutable-borrow counter was unsafe for the lens: Cambium borrows
on build/rebuild, so it could stale a LinkForm just by rendering. The final
private view lens captures an event source witness and compares after message
handling; pure build/layout does not increment. The final harness explicitly
opens form -> layouts -> chooses target -> layouts -> inserts, and passes. The
public raw input_mut path remains conservatively invalidating (including the
desktop focused-text-slot keyboard integration); normal selection-only session
intents are stamp-neutral. Actual edit, undo, redo, reload, and address-changing
SaveAs invalidate stamps. Returning to identical text or an earlier SaveAs
address cannot revive the previous proposal/form.

Final results:

- document-tests.log: standalone library 51 passed, one existing ignored.
- document-msrv-tests.log: same suite 51 passed on Rust 1.97.1, one ignored.
- document-engine-tests.log: engine library 64 passed, one ignored.
- link-tests.log: 10 existing/extended link-form model cases passed.
- link-ui-tests.log: four desktop harness cases passed, one parent helper ignored;
  the helper actually passed once in each of TWO independent child processes.
- workspace-link-ui-test.log: existing menu/layout/focus scenario passed (one).
- document-lib-clippy.log: strict library Clippy passed.
- document-clippy.log: strict test-target Clippy blocked by unchanged baseline
  useless_format in src/hard_wrap.rs:352 and items_after_test_module in
  src/writer.rs:297. No cleanup was made there.
- Desktop compilation retains baseline unused path in knot-editor/directory.rs:53
  and linker unwind-section size warning. desktop-clippy.log records scoped lint.
- Formatting, diff, and lock source/version preservation checks passed.

The two-file workflow creates real local Unicode Djot files and a read-only
link target. It applies a named relationship via palette/form, verifies no write
before explicit Save, undo/redo, link parsing, and exact saved source. A child
process reopens it and parses the relationship. Another workflow edits source
externally after insertion, verifies Save refuses and keeps the draft, and a
second child process verifies the external bytes are intact and clean on open.
These are source/model/UI-harness and fresh-process file qualifications. No
native app screen observation or complete product acceptance is claimed.

Dependency/publication caveat:

The consumer manifest uses canonical Git branch main for mere-source-edit0.1.0.
The supplier is uncommitted, so every positive command uses ONLY an explicit
qualification patch for this new crate. Normal unpatched offline Git resolution
fails: no matching package named mere-source-edit (git-resolution-unavailable.log).
The workspace lock's new row represents the local patch; it is not a published
supplier lock qualification. Existing locked versions/source identities are all
preserved. A stopped initial resolver attempted a broad refresh; neither that
closure nor its incomplete test run counts as qualification. Both locks were
restored to baseline package closure plus the one new package and final checks
used --locked --offline. The standalone lock is Git-ignored. Before publication,
resolve and qualify the published supplier normally without the patch.

Command template (from Knot checkout):

    cargo test --locked --offline -p knot-desktop --test document_links_ui \
      --config 'patch."https://github.com/merely-made/mere.git".mere-source-edit.path="/Users/markik/Code/repos/mere/crates/system/source-edit"' \
      -- --test-threads=1 --nocapture

Other focused commands use the same patch and --locked --offline:

    cargo test --manifest-path crates/knot-document/Cargo.toml --lib
    cargo test --manifest-path crates/knot-document/Cargo.toml --features engine --lib
    cargo +1.97.1 test --manifest-path crates/knot-document/Cargo.toml --lib
    cargo test -p knot-desktop --lib link_workflow::tests
    cargo test -p knot-desktop --lib document_link_menu_keeps_selection_and_cancel_escape_restore_source_focus
    cargo clippy --manifest-path crates/knot-document/Cargo.toml --lib -- -D warnings

qualification.json records exact changed paths and hashes; before/current lock
snapshots support independent package-source comparison. Testing artifacts are
outside the repo, and Mere/Isocosm files were not edited by this consumer task.
