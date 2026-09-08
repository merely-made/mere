# R1 drive and wake experiment receipt

Date: 2026-09-08

## Baseline

- Genet inspected revision: `ee0b314b3e9ac4a2fadb07fb7816990fa3f2b71d`.
- The Genet checkout had concurrent edits in `components/genet-scripted`,
  `components/script-runtime-api`, and related harness files. This receipt does
  not treat them as a clean behavioral baseline.
- `DocumentSession` supplies `pump(now_ms)` and `settled()` at
  `components/shared/document-session-api/src/session_engine.rs:852-858`.
- Ortet calls `session.pump(now_ms)` before `frame()` and requests another
  redraw while `!session.settled()` at `ports/ortet/src/shell.rs:382-467`.
- The runtime supplies `next_timer_delay`, `pump_workers`, and
  `has_worker_work`; dropping a runtime calls worker shutdown. The inspected
  Livery document pump advanced timers, microtasks, DOM capture and GC, while
  its pending-work predicate was timer based. This is source inspection, not a
  claim that a production Livery session has worker integration.

## Disposable model

Command:

```powershell
& .\scratch\pillar-research-20260908\execution\r1_drive_wake_model.ps1
```

Result: `R1 drive/wake model: PASS`.

The model asserts:

1. Immediate runnable work, a host-domain deadline, and two external sources
   coexist in one demand report.
2. A `settled`/unsettled poll collapses a deadline-only session and an
   external-only session to the same answer. Negative controls confirm neither
   fact is invented by the other.
3. A monotonically changing demand revision lets a host reread after a
   completion races wake registration and drive immediately. The model now
   separates `query`, `register`, and `recheck`: its negative control completes
   while no callback is registered and demonstrates that callback-only logic
   sleeps with runnable work. The recheck sees the changed revision and returns
   `drive-now`.
4. Navigation cancellation plus a session generation refuses an old
   completion. A current completion is accepted once; its duplicate is
   refused.

Model limit: it proves only that these facts cannot be faithfully represented
by a mutually exclusive wake classification. It does not prove a Genet API,
actual event-loop timing, worker teardown, or native presentation.

## Focused runtime test attempt

Attempted command, isolated from shared target artifacts:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\mark_\Code\scratch\pillar-research-20260908\execution\target-script-runtime'; cargo test -p script-runtime-api --test worker -- --nocapture
```

Result: stopped without compilation or test output after Cargo reported
`Blocking waiting for file lock on package cache`. A concurrent workspace build
held the package-cache lock. The process was interrupted rather than broadening
the experiment or competing for the shared cache. This is not a pass or fail.

## Decision

The next research contract should be an additive *set/report* of drive facts,
not an enum and not a universal scheduler: runnable-now, optional deadline in
the caller's time domain, and outstanding external work may coexist. It also
needs a monotonically observable revision or equivalent registration protocol,
plus session generation/cancellation semantics for late completion refusal.

Keep timing sources, actual wake registration, external I/O ownership, product
simulation clocks, and GPU completion with their respective hosts. Before any
implementation slice, rerun the existing `script-runtime-api` worker suite
after the cache is free, then add the real retained-session/Ortet O5 fixture
for navigation and worker completion.
