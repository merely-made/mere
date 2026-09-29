# Apparatus core and Mesquite attachment gate

**Date:** 2026-09-29
**Evidence level:** automated source tests, reported by the coordinating lane.

The shared primary Mere source passed these focused Cargo gates using the
existing `C:\t\cargo-targets\mere` target. The core-only gate used its
`apparatus` child target while the main target had an active build owner.

| Gate | Passed | Scope |
|---|---:|---|
| All-feature library tests | 74 | 17 Apparatus, 19 Mesquite, 38 UX |
| Apparatus core, no default features, serialization enabled | 14 | Renderer-independent contract after test-file split |
| Host integration | 75 | 16 host library, 24 scenario, 16 mere-view, 19 Mesquite |

Commands, from the Mere repository, with the target set explicitly:

```text
cargo test --offline --locked -j 2 -p mere-apparatus -p mesquite -p ux-events --all-features --lib
cargo test --offline --locked -j 2 -p mere-apparatus --no-default-features --features serde --lib
cargo test --offline --locked -j 2 -p mere-view -p mesquite -p cambium-genet-winit-host --lib --test scenario
```

Core and host logs are retained at
`C:\Users\mark_\Code\testing\cambium\apparatus\core-serde.log` and
`C:\Users\mark_\Code\testing\cambium\apparatus\host-integration.log`.

The gate groups overlap; these counts are not distinct coverage totals. Core
tests cover disabled retention, count/byte/age bounds, accounting overflow,
independent readers, rejected/dropped sequence holes, pagination, reset and
backward-time rejection. UX tests cover zero-capacity recording with continued
observer fanout and finite channel eviction. Mesquite tests cover optional
attachment compatibility, supplied gaps/loss/correlation, receipt sampling
ordinal and attachment failure. The host gate checks existing scenario and view
integration after the change.

The README example was not registered as a library doctest: its command ran zero
tests and contributes no additional receipt.

Turnstone's separate redacted event-copy adapter and Gloss/Inspector pane
migration are written but await dependency pins and their own tests. The actual
worker pilot waits for Redshank's active owner to finish its podcast/task commit.
No new headed diagnostics or human AT evidence was produced by these gates.
Exact captured-frame/state correlation, worker acceptance versus execution
versus durability, and two-consumer qualification remain open.

See the [canonical diagnostics plan](../implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md)
for current ownership and remaining acceptance conditions. June receipts remain
historical evidence.
