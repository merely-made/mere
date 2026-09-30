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

## September 30 bounded continuation

The shared inspection data now projects independent batches into bounded
read-only rows, showing retention, each loss class, omitted display rows,
scoped supplied causes and explicitly unavailable correlation. Products supply
already-redacted borrowed payload labels. The structural UX projection does
not invent layout bounds, focus or actions. The all-feature Apparatus library
gate passes 22 tests; its no-default-feature serialization gate passes 18.

Registry ingress now uses a bounded nonblocking staging channel. Count,
accounted encoding bytes and age are configurable; any zero disables buffering.
Full queues reject newest attempts. Contention, poisoning, disconnection and
no-sink emissions have observable loss. Unsequenced attempts are reported as
unsequenced loss rather than fabricated Apparatus gaps. The keyed invariant
matcher bounds its combined pending pool and uses supplied run/source/operation
identity. Deadlines are swept before terminal matching, so a late completion
cannot erase a timeout. Legacy uncorrelated matching remains explicitly FIFO.

Registry's default all-targets gate passes 135 tests with one existing ignore;
the restored diagnostics-only feature gate passes 36. Replacing keyed matching
with FIFO, and moving deadline sweep after terminal delivery, each fail the
regression with exit 101. Both source controls are restored. Logs are under
`Code/testing/mere/receipts/registry-*.log`. These bounds cover staging and
pending tracking, not producer allocation, runtime descriptor/orphan catalogs
or an installed application pipeline.

Mesquite can opt into an immutable product reading taken after queue presentation
and before queued pointer/platform accessibility delivery. A separate backend
packet carries the run/request and host/frame/dimensions/scale identity with
the pixels; receipt collection requires an exact match. Legacy no-observer
products retain their receipt shape. Metadata limits default to 128 requests,
64 KiB per projection and 1 MiB total, with one pending projection. Products
retain redaction, bounded construction and revision/cause authority. These
limits do not cover pixel storage, PNG files or legacy receipt fields.

The focused native/shared gate passes 64 tests (host library 16, existing async
capture one, three new pairing suites, scenario 25 and Mesquite 19). Synthetic
delayed pixels establish pairing mechanics, not a GPU receipt. Restoring the
late-state sampling defect changes the captured count from 11 to 66 and fails
with exit 101; the source is restored and the full focused gate passes again.
The actual WebCapture backend compiles for `wasm32-unknown-unknown`; browser
interaction is not claimed. Logs and hashes live under
`Code/testing/cambium/capture-pairing/`. These commands use Rust 1.98.1,
locked/offline dependencies, two jobs, zero dev/test debug and the reused
`C:/t/cargo-targets/mere`. Final containing pins and product native qualification
follow separately; no completed consumer receipt is claimed by this section.

Presentation stamps identify the rasterized source queued for presentation.
They do not certify GPU completion, compositor visibility or physical display
acknowledgement. Turnstone's custom compositor requires its own capture
adoption. Human AT, broader live-document/custom-leaf semantics and a second
real-worker causal flow remain open.

### Containing Genet pin, September 30

Genet `7bf0e448a5c395cb5e205b73bcb277c73472e8c6` is published with 28
Taproot tests, 40 render tests and a restored scope-preflight failure control.
Mere's root and standalone Graphshell web manifests now name that revision.
Genet changed no manifest or lock between Mere's previous `c5470fcb` pin and
this revision. The root Mere lock is byte-identical to its saved baseline after
only substituting that Git identity; package versions and dependency edges are
preserved. Locked/offline containing execution validates this lock.

The containing native gate passes 108 tests: the same 64 host/Mesquite tests
plus all 44 Rootstock library tests, including existing GPU producer checks.
`containing-7bf.log` records the command and results. The actual web host again
passes Wasm compilation at this revision (`web-wasm-7bf.log`). Standalone
Graphshell's ignored lock needs its existing Gazette `async-trait` dependency
edge in addition to the Genet identity substitution. Its actual port check and
subsequent locked/offline Wasm check pass (`graphshell-web-7bf-resolve.log` and
`graphshell-web-7bf.log`); `containing-context.json` records both lock hashes.
Compilation does not
qualify browser runtime or human accessibility.

### Combined Commands/fonts integration, September 30

Knot's concurrent Commands, links and typography line used forked shared pins,
rather than descendants of the diagnostics cohort. Genet's clean merge is
`69a2383b2ad777b884a72f31f8f8fb7ece275c0b`: two font-backed host length tests,
28 Taproot tests and 40 render tests pass. Its four semantic source files remain
byte-identical to the preceding semantic receipt and its lock is unchanged.
Mere merges the command menu/key route and adopts that combined Genet identity
while retaining Gaz, Registry, Apparatus and presentation seals. The root lock
changes only Genet source identity, preserving all packages and dependency edges.

The broader containing all-targets gate passes 210 tests, followed by seven
command-menu tests and five platform-key tests. The containing locked/offline
rerun passes after Cargo materializes the fresh Git checkout. The earlier two
offline attempts stopped at an uncached source, before compiling; their logs
remain beside the locked online cache/build and passing offline receipt. Actual
web-host and standalone Graphshell Wasm checks also pass locked/offline.

Scoped Clippy retains explicit baseline qualifications. The broader host
all-targets check adds one unchanged `input_routing.rs` type-complexity fixture
to the preceding three baseline allowances. The Cambium/platform library check
reports 17 warnings in five byte-identical baseline files, requiring six
additional lint allowances. No warning site is in the incoming menu/key code.
Failure logs and Git-blob comparisons are retained; these are qualified scoped
passes, not unqualified strict all-target passes. Logs and source/lock hashes
are under `Code/testing/cambium/capture-pairing/combined/`.

Knot and Redshank must qualify the combined pins before Turnstone's containing
gate. Their earlier native captures retain their original source/binary hashes.
New Windows Knot receipts must exercise the actual client-menu default; legacy
plain-row harnesses cannot establish that default's native behavior.
