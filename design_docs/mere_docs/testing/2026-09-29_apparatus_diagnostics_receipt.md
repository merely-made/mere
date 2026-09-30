# Apparatus core and Mesquite attachment gate

**Date:** 2026-09-29; combined consumer continuation 2026-09-30.
**Evidence level:** original focused source tests below, followed by qualified
shared/consumer source tests, actual Wasm compilation and reviewed native
captures in the dated continuations. Human AT acceptance remains open.

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
adoption. Human AT and broader live-document/custom-leaf semantics remain open.
At this shared-core stage the second real-worker domain was still pending;
the combined consumer receipts below qualify Knot's catalog domain without
establishing whole-application causal authority.

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

The published containing Mere revision is `bd5912fbbb8f468defc3bbeee7eac5a4f7d2b2f3`.
Knot and Redshank now qualify these combined pins before Turnstone's containing
gate. Their earlier native captures retain their original source/binary hashes.

Knot `3dfb70b01e79dadfcbd1e615ded43b34f01802da` preserves Commands, links,
typography and diagnostics. Its desktop all-target gate passes 260 with one
existing ignore; standalone document all-features passes 63 with one existing
ignore. Production Clippy is strict-clean; the all-target pass retains its six
verified test-baseline allowances and archived strict failures. A deliberately
bypassed stale-result acceptance guard fails 101, then exact source restoration,
seven graph-module tests and the build pass. Five native runs exercise actual
client menus and produce nine reviewed images: accepted Save, explicit partial
read errors, count-one loss and paired/default light/dark/light. Schema v2 seals
the actual measure enum. Source, binary and receipt hashes are in
`Code/testing/knot-editor/catalog-diagnostics/combined/final-audit.json`.

Redshank is repinned at Woodshed `cefc903dcd7f506803acfe2ee52cad7693787188`.
All 45 tracked Redshank Rust source blobs remain identical to `2479dc9`; only
manifests/lock changed, with all existing package versions and dependency edges
preserved. Its 68 desktop tests, strict production Clippy, native build and
actual Wasm compilation pass. Six new paired/default native images were reviewed;
paired frames 22/25/28 seal dirty and durable revisions 3/4/5, with explicit
two-record loss. Three corrupted metadata controls reject the mismatch. The
primary's 11 transcript WIP files are preserved, and the collision worktree and
branch were removed after publication. Evidence is in
`Code/testing/redshank/capture-pairing/combined/`.

Knot seals supplied accepted-catalog references only when that graph is visible.
Redshank seals rendered UI facts alongside separately named persistence owner-time
state and admission context; it does not assert that a Save caused those pixels.
These qualify product-owned catalog and persistence domains alongside submitted
frame/readback pairing. They do not establish whole-application causality,
semantic revisions, human AT or compositor/physical-display acknowledgement.

### Published combined Turnstone consumer, September 30

Turnstone `b2ead70a448948a1e8a9bde10b34f01524119606` consumes Mere `bd5912fb`,
Genet `69a2383b`, Knot `3dfb70b` and Woodshed `cefc903`. The final lock preserves
preceding registry package versions; intended Git identities and the direct
document-session-api edge change. Locked/offline all-target tests pass 635 with
nine existing ignores; four additional harnesses run zero tests. Five exact
optional Piccolo/Wasm participant tests each pass, independently of the default
native build. These are native runtime-feature tests, not a Turnstone browser or
Wasm-target receipt. Focused groups overlap the workspace and must not be summed.
Turnstone strict Clippy is not claimed.

Scoped contributed automation and platform actions use the same provider-owned
computed semantics and retained rectangles. Targets carry pane/admission/node
identity; the held pointer revalidates after scrolling and rendering. The real
Sky scenario passes: Calculate request 3 moves from painted y=839 with no visible
rectangle to the full 30-pixel visible rectangle at y=570, before the following
applied-state assertion. Three 1024×600 images were reviewed. Its final scrolled
background is black and field values are unreadable; product state is established
by the scenario observation, not screenshot text.

Final source review discovered an independent platform admission bug: a surface
could refuse hidden/disabled/unsupported actions while the helper reported true,
allowing rejected Focus to move Shell focus/stacking. The real-runner regression
fails before repair (101); after returning `Option<SurfaceRequest>` and using
`.is_some()`, all eight contributed AT tests pass. A test-provider wrapper
qualifies admitted no-host-effect actions; production Runner redraw is preserved.
Dropping the final generation predicate fails 101, then exact source SHA-256
`6E4DCB660590EB17BAE88BF6668363134319E73AFDBDDDC7C6787273FA3FB3AE` is restored.
Removing the Gloss platform arm also fails its control and restores exact bytes.

Optional Gloss Diagnostics fresh and restored scenarios pass with two further
reviewed 1024×600 images. Fresh retains two records/266 accounted bytes, four
evictions and gap `[1,5)`; restart retains one record/133 bytes with no loss/gaps.
Saved Diagnostics, Downloads order survives restart. Explicit display limits
16/4/256 are recorded in both launches; readonly rows show unknown coverage and
unavailable correlation. Default Gloss remains minimap plus Downloads.

The final audit verifies 254 input hashes unchanged through qualification and
publication, with all native processes closed. Binary SHA-256:
`DFBE44B79470C2C32EF0A2EDD0BBCCF78EAE4D3861B876D8E91B29AAAFD7A09F`.
Lock SHA-256:
`7C45B1754D9C3A9557A1BB56882F5CB481F514F1228F2EFD3E3E2CC0D94BBD94`.
Evidence lives in `Code/testing/turnstone/contributed-semantics/` and
`Code/testing/turnstone/diagnostic-inspection/final/`; publication.json records
the remote identity and root review. The unrelated `.github/` tree is preserved.

These captures qualify behavior/visible readings, not an immutable Turnstone
diagnostic-state/pixel seal. The custom compositor, Inspector source/endpoint
binding repairs, broader built-in/custom/live-document surfaces, additional
worker families and human AT retain owner-specific gates. The recorded Windows/
Narrator baseline is preparation only and supplies no human acceptance evidence.
