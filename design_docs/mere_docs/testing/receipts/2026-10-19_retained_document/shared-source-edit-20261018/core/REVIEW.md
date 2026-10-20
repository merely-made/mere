> Copied 2026-10-19 from the M4-local `~/Code/testing/shared-source-edit-20261018/core/REVIEW.md`; see the [receipts index](../../README.md).

# Independent contract review

Scope: shared `mere-source-edit` contract, tests and ownership documentation.
No blocking correctness issue found in sorted base-coordinate edits, UTF-8
validation, overlap/same-start refusal, result coordinates or independent
document/instance/revision/exact-source guards. Candidate-versus-publication
wording was judged explicit. Consumer review was limited to the candidate
application boundary, not a full product audit.

One documentation finding was resolved: `edits()` returns ascending
base-coordinate ranges, not an ascending sequential mutation program. Shared
API docs now require whole-batch/one-undo semantics, descending-offset replay
or equivalent simultaneous semantics, and refusal without source/stamp/undo
changes. Hosts lacking a batch undo boundary must accept only one range.

The reviewer did not duplicate test execution. Parent-run tests, strict Clippy
and Rust 1.97.1 logs are separate artifacts here. This does not establish native
GUI or publication acceptance.
