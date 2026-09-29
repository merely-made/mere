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
migration are published at Turnstone `d6b62adbd2929e46bde5611bf11f01a34eeeaceb`
and qualified on the sealed graph: 612 workspace tests with nine ignores,
five participant checks, 33 UI checks, a normal native build and final fresh/
restarted scenarios pass. Initial retention of two records reports eight evictions
and gap `[1,9)`; configured Downloads/Recent order and Inspector persist. The
consumer receipt records all three inspected nonblank captures and source/binary
hashes at `turnstone/design_docs/2026-09-29_shared_diagnostics_gloss_inspector_receipt.md`.
Separately, Redshank's persistence pilot
is published at Woodshed `a57085b`: 63 desktop tests, strict Clippy, a restored
early-durability negative control and native opt-in/default/invalid-setting
receipts pass. Its bounded native batch retains two records, reports 16 evictions
and a `[1,17)` gap, and links successful execution to the save reply that advances
durable revision to 3. See
`woodshed/design_docs/2026-09-29_redshank_persistence_diagnostics_receipt.md`.
These are separate consumer gates, not additional coverage from the Mere commands
above. Human AT, exact captured-frame/state correlation, other worker families
remain open. These two products qualify the bounded core; they do not establish
two complete action/worker/exact-pixel causal receipts.

See the [canonical diagnostics plan](../implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md)
for current ownership and remaining acceptance conditions. June receipts remain
historical evidence.
