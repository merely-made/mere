# Hest qualification follow-through, 2026-10-21

## Published-package MSRV

Fresh Linux qualification on Rust **1.97.1**: **324 tests passed**, zero failures,
and one existing ignored Cambium doctest. This covers the four published release
archives and both registry-only compatibility fixtures, not the full Git workspace
or backend packages. No release or production source changed.

`archives.json` records the freshly downloaded crate archives and checksums,
verified against the prior publication proof. The extracted packages were tested
with their shipped locks using `cargo +1.97.1 test --locked --all-features`:

| Package / receiving fixture | Passed | Ignored |
| --- | ---: | ---: |
| Hest 0.1.0 | 13 | 0 |
| Hest 0.2.0 | 17 | 0 |
| command-menu 0.1.1 | 0 (build and doctest harness) | 0 |
| Cambium 0.5.0, all features, integration and doctests | 289 | 1 |
| Published Cambium 0.4 / command-menu 0.1.1 / Hest 0.1 | 3 | 0 |
| Published Cambium 0.5 / Hest 0.2 | 2 | 0 |

The compatibility fixtures resolve registry dependencies with Rust 1.97.1, then
run locked. Their exact locks are retained separately in `locks/`, along with the
four package locks. `checks.json` records every command, result, timestamp,
resource wrapper and compressed log. `msrv-environment.json` records the compiler,
platform and original lock hashes. Zero-test shim compilation is not counted as a
behavioral test; the old registry fixture verifies its cross-crate type identity
and stored-choice compatibility.

Each gate acquires `testing/machine-heavy.lock` and runs under a systemd scope:
MemoryMax=3G, MemorySwapMax=0, CPUQuota=150%, one Cargo job and test thread,
incremental/debug info disabled, nice 10, 600-second timeout. The wrapper is
`gate.py`. Peak RSS in each log is process accounting; the systemd limit is the
resource boundary.

To reproduce, download/extract the archives listed in `archives.json`, compare
SHA-256 before testing, and run the package command above in each extracted
package. Copy each retained fixture lock into its fixture as `Cargo.lock` and run
the same command there. Apply the recorded resource wrapper. Preserve a new
output directory for each attempt.

## Headed browser: limited pass, audit export open

At source `c66620cae016a856bd8cdae7dca189293bf295b6`, the headed Linux X11
Chrome 155.0.8059.84 WebGL2 viewer scenario passed with zero page errors and zero
gate failures. It exercises Hest `Invocation` envelopes through Mesquite's
`invoke`, observes the zoom model changing, then exercises the retained menu,
disabled-command feedback, search, synthetic keyboard, pan/wheel policy and
focus-return paths. This is automated input, not physical keyboard or human AT
acceptance. See `scenarios/hest_viewer_commands.scn` and `viewer-runtime/`.

The menu and disabled-command captures were visually inspected: command names,
search and “Select a visible node first” are readable. The initial invocation
and final frame show no visible graph nodes; this packet does not establish
general graph-scene presentation correctness. The receipt reports no adapter
identity and no GPU timestamps, so it does not establish physical GPU identity.

**H4 browser audit export remains open.** The successful exported result contains
no `commands` records. Mesquite's `receipt_value` and `text_receipt` include its
command audit, but `Product::complete` receives only `taproot::Outcome`.
Graphshell's `TreeLane::complete` constructs a separate report, and the browser
`publish` adapter omits the audit. The actual command receiving path ran; its
terminal audit did not reach the browser receipt. These adapters remain unchanged
at the publication base below. Closing this gate requires exporting Mesquite's
existing records and rerunning the browser scenario with assertions on the
invocation, outcome and observed change. It does not require an H5 phase.

The viewer uses the official read-only feature set,
`--no-default-features --features presentation-webgl2,without-classic-vello`,
Rust 1.99.1 and dev opt-level 1/debug 0. It packaged successfully under the same
3 GiB cap. This is not a release artifact or size-budget qualification.
`packaged-viewer.json` identifies the served Wasm by SHA-256; the environment
receipt independently records the same hash. The browser process shared the
runner's capped systemd scope. `headed-runner.patch` records the local harness
change to launch Chrome directly in that scope using X11/ANGLE GL.

Full default-feature Wasm compilation succeeded at dev opt-level 0 and 1, but
packaging attempts were stopped during sustained reclaim at the 3 GiB limit.
Removing only the optional Wasm name section did not resolve that full-product
packaging limit. The streaming strip script and section hashes prove all other
sections were retained byte-for-byte. The narrower viewer did package and run;
the full-product browser and saved-choice reopen gates remain unqualified.
`hest_commands_save.scn` and `hest_commands_reopen.scn` are prepared but **unrun**.

`browser-attempts.json` retains all 14 build, strip, packaging and runtime gates,
including interrupted attempts. `superseded-attempts.json` explains lock waits,
timeouts and deliberate stops. Log hashes refer to decompressed bytes; logs are
stored as gzip. Builds that timed out were resumed from Cargo's completed cache
under the same limits. No limit was raised.

For reproduction, use the frozen source above and `locks/graphshell-web.lock`
for the standalone web workspace. Recreate its web site, copy `harness/` into
the site's `tests/` and `scenarios/` into its scenario directory, and follow the
recorded viewer build/strip/bindgen commands. The served package has `pkg/webgl2`
pointing to `.`. Supply the current display authentication and Chrome/Node paths
when running the headed harness; the recorded paths describe this machine.
Raw Wasm, generated JS and the isolated browser profile stay local rather than
being committed. Package, source, harness and scenario hashes are retained here.

## Source and remaining acceptance boundaries

Upstream was pulled before building and again before publication, reaching
`b4ae63d8234c9a6443060249023e394124b58348`. Hest, the command-menu shim and
Mesquite's command lane/API did not change between these commits. Other host code
did change, so the browser runtime claim remains at frozen `c66620ca`, not the
new tip. The published-archive MSRV result is independent of that host drift.
`qualification.json` records these boundaries. This packet changes evidence and
the plan only; there are no production changes requiring downstream pin updates.

The native resident Graphshell path routes window controls through Hest
invocations. Its existing full native/cold-reopen controller is
`ports/graphshell/docs/receipts/coop_graph_view_20261020/run_native.py`, whose
launcher uses macOS app bundles. That prior receipt is not a fresh Linux Hest
acceptance. Native window-manager behavior, physical keyboard/IME and human
assistive-technology acceptance remain separate.

Human AT acceptance must name the actual host, OS and screen reader, exercise
focus and announced command names/states, invoke a command through AT, and observe
the receiving host's result. Automated semantic-tree checks cannot substitute for
that exercise. Separate backend package releases remain outside this packet.
