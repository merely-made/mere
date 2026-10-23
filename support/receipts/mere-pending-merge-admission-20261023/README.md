# Pending-merge admission parity — 2026-10-23

The live carrier and complete materialization share zero-or-one pending-merge card admission, first-media lookup, and JSON/schema decoding. Multiple cards and a missing first media resource close the route and purge staged state. A card with extra media still uses only its first resource.

The tests-only carrier run reproduced both malformed-owner-state gaps: one healthy control passed and two refusal assertions failed (`carrier-red-r2`). The initial compile attempt used the wrong fixture InstanceId integer type; it was corrected before behavioral red. The first repair compile missed a schema constant qualification; `resident-green-r2` is the successful final gate. These earlier failures are retained.

Final focused gates, debug information off and incremental compilation off:

- Full resident library suite: 27 passed, 0 failed.
- Wire-only `--no-default-features` check: passed.
- Independent source review: no concrete issue found in admission/failure cleanup.

This qualifies malformed owner-state admission and synthetic carrier cleanup. It does not assert that normal producers emit malformed cards or qualify native UI/transport behavior. No persisted grammar, checkpoint, raw-term execution, or render dependency changes.

Logs and exact commands are adjacent. `source-sha256.json` binds the final implementation/test files. Parent source: 4fae818ad3a3ffc3f802697732de8ace10bcaeb7.
