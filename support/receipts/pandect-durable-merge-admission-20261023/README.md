# Durable merged-marker transaction admission — 2026-10-23

A two-handle regression reproduced the stale-marker gap: target undo/persist, open source while editable, target redo/persist, then `transact_deltas` through the held source returned `Ok(Applied)`. Fresh-open refusal and undo-without-redo write controls passed (`marker-red`).

The existing candidate-write transaction now reads durable `merged.json` alongside its log-head checks. Any present marker refuses with `SessionError::ReadOnly(session_id)`; a marker-read error refuses without writes using the existing validation error form. The candidate is installed only after successful transaction acknowledgement with no admission rejection. No public API or persisted format changed.

Final full Pandect library gate: **340 passed, 0 failed** (`library-green-r3`), debug information and incremental compilation off. The stale source refuses both ordinary and published transactional edits. Each refusal leaves the entire MemoryBackend key/byte map unchanged, alongside unchanged held/reopened graph/history and retained merge marker. Undo without redo remains writable; fresh reopen and explicit marker reload retain read-only behavior.

The first two full runs passed 339 tests and failed the new published control before transaction admission because its fixture lacked qualified source placement. A recorded parent alone was insufficient: branch creation uses a fresh unqualified session. The final fixture reopens and qualifies the durable source through the existing legacy receipt API, then explicitly preflights addressable publication before merge. Earlier failed evidence is retained. The original tests-only red patch is retained separately.

This repairs the atomic transaction APIs for independently held source handles. Djinn's registered-host undo/redo already reloads the source marker. This does not claim a normal registered Djinn incident or change the legacy synchronous edit/flush lifecycle; checkpoint design remains separate. A separate source review found no concrete atomicity issue.

Exact commands/logs and final source hashes are adjacent. Parent implementation commit: ddef74811.
