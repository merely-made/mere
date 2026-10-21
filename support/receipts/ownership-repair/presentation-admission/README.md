# Shared presentation admission

This successor closes Q067/A064's structural admission split. The sealed audit registers and their original mismatch probe remain unchanged.

`chirograph::validate_presentation` owns the scene/presentation relationship: bindings name active items, each item has at most one binding, and each codec names its required capability. Capture V1/V2 now use that owner before encoding, decoding, content addressing or resource checks. Graphshell's snapshot, diff and persisted-session restoration use the same checks through their existing admission boundary.

Live editable-text retention and purge policy remain in Graphshell. Captures omit session cache policy by design. Resource availability remains a caller assertion; this change does not authenticate source authority or verify resource bytes. Missing offers and orphan offers retain their prior admission semantics.

The public-API regression in `crates/graphshell/graphshell-client/tests/presentation_admission.rs` adapts the original Q067 probe's valid control and three malformed cases to the repaired expectation. It also covers tombstoned bindings, both capture wire versions and V1-to-V2 decode, and rejection before resource lookup. The client regression checks transactional rollback on a malformed presentation diff. Existing package tests cover retained scene/diff/resource handling, persistence and portable projection realization.

Scope is portable presentation admission. Q068's UTF-8 reader disagreement, graph-history ownership, receipt classification and browser/network/platform qualification remain outside this cut.

Verification: **151 passed, zero failures/ignores**, across Chirograph and Graphshell client default-feature library/integration suites. The initial offline attempt stopped on an uncached upstream supplier; the locked online retry passed without changing `Cargo.lock`. Exact source/lock/probe hashes, commands, resource limits and logs are in [qualification.json](qualification.json). These are local native API and realization tests, not browser or network qualification.
