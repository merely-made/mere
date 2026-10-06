# Pre.4 remote allocator diagnosis

Date: 2026-09-30. Status: measured diagnostic; repair not selected or accepted.
Source: `124fc42bb4809dd27f9029d91e966915c97d5fd3` plus temporary fixture-only
post-failure instrumentation, restored afterward. Production patches,
dependencies and acceptance thresholds were not changed.

## Authority and method

Isometry ruling 411 records Mark's exact answer **"A!"** to bounded diagnosis,
retaining zero baseline and returning ownership/patch-design changes as forks.
Canonical ruling commit: `052ee05`. Exact question and answer companions remain
in the external `post-retirement` evidence directory.

The original gate uses 400 samples with nominal 10 ms delays. Only after it
fails, the diagnostic adds four 100 ms samples without explicit sync, existing
`client.sync().await`, immediate/100 ms measurements, then `memory_cleanup`
and another sample. Allocator observations can submit queued work; they are
not passive device telemetry. Diagnostic capture errors are logged as Results
without replacing the original error. Both runs use the same executable and
six-file MiniLM model, 512-row cancellation, Rust 1.98.1, plain WGPU without
fusion/autotune, and an offline locked release build in the reusable Mere
target. The control removes the diagnostic environment flag. Each run has a
120-second bound; both exit 1, stdout empty, retaining the original error.

## Measurements

| Observation | Active allocations | Active bytes | Reserved bytes |
| --- | ---: | ---: | ---: |
| Baseline | 0 | 0 | 0 |
| Original failed receipt | 10 | 5,323,776 | 41,943,040 |
| Same-executable control failure | 10 | 5,323,776 | 41,943,040 |
| Diagnostic original failure | 10 | 5,323,776 | 41,943,040 |
| Each of four samples without explicit sync | 10 | 5,323,776 | 41,943,040 |
| After explicit completion wait | 0 | 0 | 41,943,040 |
| 100 ms after completion wait | 0 | 0 | 41,943,040 |
| After subsequent cleanup | 0 | 0 | 0 |

Completion wait returned `Ok(())` in **1.9222 ms** in this run. This is not a
latency guarantee. Reserved bytes are allocator accounting, not physical VRAM.

## Reading, not ruled

Completion polling made these buffers releasable. Source tracing shows cleanup
can submit other streams' work after the worker's current-stream wait. WGPU
releases cross-stream pins through completion callbacks and parks its polling
thread without an active owner. This supports a completion-ordering explanation,
not exhaustive identification of every retained handle. Detached readback and
transfer ownership still require repair controls. The old pre.2 receipt lacked
allocator counters, so a regression from pre.2 is not established.

The proposed burn-remote repair is a pending patch-design fork. Acceptance must
keep zero active allocations/bytes, prove cancellation and exact fresh-session
recovery, preserve an unrelated live lease and its tensor values, and propagate
injected sync failure rather than acknowledge clean closure. A device-wide wait
can wait on other work. The separate native-reference finiteness verifier and
strict numerical ceiling also remain required. These failed runs produced no
completed JSON numerical receipt.

## Verification and evidence

Independent review checked source/run/log/executable/model hashes and exact
restoration. Nine deliberately corrupted inputs were rejected: false success,
timeout, control diagnostic flag, empty log, changed retained count, sync error,
missing cleanup observation, nonzero post-sync active bytes and unexpected
stdout. This accepts diagnostic evidence, not the migration.

Diagnostic executable SHA-256: `35404dfcbb760100303090ee86b2c6572a5eba89a901b59d1e5d3cdd59b77f12`.

- [2026-09-29_pre4_allocator_stop.log](../../../ports/distillery/probe/receipts/2026-09-29_pre4_allocator_stop.log): `f4af9b13919a7eaeeb1a89bf66fb73e847cac987b5175fffc5b22fe45bf73d68`.
- [2026-09-30_pre4_allocator_control.log](../../../ports/distillery/probe/receipts/2026-09-30_pre4_allocator_control.log): `f4af9b13919a7eaeeb1a89bf66fb73e847cac987b5175fffc5b22fe45bf73d68`.
- [2026-09-30_pre4_allocator_completion_poll.log](../../../ports/distillery/probe/receipts/2026-09-30_pre4_allocator_completion_poll.log): `bcbc8db6a5a457b5ac444658743f44006ab2ffd646c40e4bdb3b4df6673df515`.

Raw logs use exact-path `-text` attributes. Full command/source/model seals,
candidate, review and restoration receipts remain under
`Code/testing/mere/receipts/2026-09-29/pre4-s13/allocator-diagnosis`.
Original source bytes, timestamps, configuration and clean status were restored.
The generated executable still contains diagnostic code and must be rebuilt
before an acceptance run. The existing migration worktree and shared target are
retained for the unresolved gate; no new isolated home, target or worktree was
created. Main integration and downstream repins remain held.
