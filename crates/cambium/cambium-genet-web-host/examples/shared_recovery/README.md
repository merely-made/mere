# Shared browser Workshop recovery fixture

Execution status is source- and gate-specific: use the coordinator's sealed
build, fixture, driver, raw-pixel and resource-closure receipts together.
It mounts two independent full production Workshop states and preview bindings
on two canvases sharing one actual `RenderCore::boot_async` core. The local
reserved Reader/Stylesheet keys are identical; the production registry's physical
GPU key remapping must keep their resources independent.

The exported `run_shared_recovery(firstCanvasId, secondCanvasId, freshRunId)`
returns a bounded JSON proof. `status()` exposes the current phase or final result.
`capture_rgba(name)` returns a tightly packed RGBA8 artifact by receipt name.
There are twelve retained copies: four complete presented frames and eight
complete actual registered Reader/Stylesheet images, before/after on each host.
Every copy is admitted at nonempty 32 MiB before readback allocation; the complete
retained before/after plan is admitted at 128 MiB before collecting source copies.
Actual final byte accounting also precedes insertion into the artifact map.
Export arrays serially and call `clear_capture_artifacts()` when finished.
The fixture runs once per module instance and uses in-memory libraries only.

The proof captures both baselines, hides the second host through the real host
visibility API, queues one actual uncollected readback, arms all three typed
capture callbacks on the hidden host, and destroys the owned device. It relies
on production browser wakes and owner recovery, without manually detaching or
acknowledging from the fixture. It requires actual old-generation `Destroyed`
refusals, queued readback `DeviceLost` then `AlreadyCollected`, old-core release,
shared new-core adoption and same-canvas/host new-generation queue presentation.
Both actual preview image identities must rehydrate. Presentation-bound seals
compare authored theme, dirty state, focus/selection/zoom, runner and source
identity. The separate full producer copies resolve the exact physical renderer
key and full registered dimensions; they never reconstruct or crop a specimen.
Reader and Stylesheet baseline bytes must each differ between the independently
authored hosts. Both full frame and full producer comparisons apply unchanged
interior RGB 3 / alpha 2 / mean RGBA 1 limits uniformly, without edge allowances.
Subsequent logical `Host::key` dispatch must change state and pixels, and CSS
resize must change the presented extent. Actual DOM keyboard gestures and OS
input delivery remain separate acceptance lanes.

The fixture explicitly provisions the outer Host and both retained production
preview documents with the checked-in Roboto face mapped to `sans-serif`,
`system-ui`, `serif` and `monospace`. This is a delivery fixture only, not serif,
fixed-width or production script-coverage fidelity. Production consumers select
appropriate actual faces for each role. Appearance/typography reconstruction and
viewport changes must retain the admitted ledgers. Each presentation seal
requires `font_resources.outer_materialized_fonts > 0`, zero outer registration
errors, and positive Reader/Stylesheet glyph counts with actual font byte hashes.
The external driver independently retains the thirteen-color nonblank gate for
every complete raw image and the six full before/after pixel comparisons.

Actual canvas and accessibility-mirror synthetic DOM `keyup Tab` controls check
the owned listeners and preserve focus/draft authority. `Host::key_released`
ends held-Tab spatial navigation; complete Tab taps precede continued slider
input. These controls are labeled synthetic DOM adapter proof, not OS keyboard
or assistive-technology actions.

Startup controls use the actual public mount acquisition/lifetime seam. A
previously disposed lifetime must never poll acquisition or call init. A second
control acquires a real core, holds its delivery, revokes authority in the same
poll that delivers it, and requires refusal before init. A caller observation
handle keeps that fixture-private device alive solely for explicit cleanup: after
Disposed/no activation, the fixture destroys it, waits for the actual matching
generation's `Destroyed` notice, drops the observation handle, and requires the
Rust core to be released. `control_device_closure` records that distinct outcome.
This does not claim cancellation of an underlying JS WebGPU promise or physical
driver reset. Activated mounts are explicitly disposed before
success returns. Copied cleanup status must show Disposed, no startup/cleanup
pending, no owned listeners/RAF or removal failures; the actual host surface,
core and recovery-owner bindings must be gone. `dispose()` and `lifetime_status()`
remain available for idempotent cleanup and observation after a failed run.
Mount status alone does not prove group-owned acquisition-task disposal.
After both mounts complete disposal and all readbacks are collected/dropped, the
fixture explicitly destroys its exclusively owned replacement device. It waits
up to five seconds for the actual matching replacement-generation `Destroyed`
notice, then drops the group owner/core and requires the Rust core's weak handle
to expire. The receipt distinguishes `actual_replacement_device_destroyed` from
host-binding release. Generic mount disposal never destroys a shared device.

This is controlled owned-device destruction and queue presentation, whose copied
raster sources require a separate browser visual review for compositor visibility.
The second host is application-hidden within a visible page; background-document
RAF suspension is separate. It does not prove physical driver reset, OS
accessibility or a measured factory invocation count. Initial boot is covered by
the finite external process deadline; operation waits and each copied readback
are five seconds, with a readback deadline measured from copy admission. The
production recovery adapter also owns its five-second async acquisition deadline.

**Current-frame asynchronous GPU error scopes and Classic shader-counter
qualification remain open.** Both success and failure receipts say so. Pending
counters are diagnostic; pixels and synchronous Wasm raster do not close those
gates. A fixture JSON success or matching raw pixels alone does not accept the
whole browser driver: console errors, bounded owned-process cleanup, lock and
artifact closure still have to pass their independent receipts.

Coordinator-only commands under the serialized Cargo lease:

```sh
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --locked \
  -p cambium-genet-web-host --example shared_recovery \
  --target wasm32-unknown-unknown -j2
wasm-bindgen --target web --no-typescript --out-name shared_recovery \
  --out-dir <owned-output>/pkg \
  <leased-target>/wasm32-unknown-unknown/debug/examples/shared_recovery.wasm
```

Copy `index.html` beside `pkg`, serve only that owned output under the server
lease, and drive the one-shot button or exported API under an explicit browser
handoff with a finite outer deadline (60 seconds). Record exact source/pin/binary
hashes, the JSON seal, all raw artifacts and visual review. Final disposal may
remove accessibility mirror nodes; use the retained raw artifacts for review.
No server, adapter, build, test or browser was launched while authoring the fixture.

## Accepted R6 evidence

The [compact browser R6 receipt](../../../../../design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/browser-r6/README.md)
records actual Genet `5618326d` / Mere `4682ba99` / Netrender `c74b7cb` /
provider `2eae624` code, reused compiled R4 Wasm/glue and unchanged R5 package,
with a separately sealed R6 ownership driver. All twelve complete raw/PNG images,
six full recovery comparisons, five CPU font-resource controls and independent
numeric/visual reviews pass. Maximum RGB difference is 1, alpha 0 and mean
1.9501730778606602e-5 under unchanged RGB 3 / alpha 2 / mean 1 limits. The actual
async owner and hidden-host lifecycle succeed; actual private devices and
mount/Chrome/Node/server resources close within their finite recorded bounds.

This is explicit font delivery/retention with the fixture's one Roboto face
mapped to four requested generics. The earlier native R2 proof uses G09b/Mcc65,
not these later retained font-source changes. R6 records pending quality
Some((3, 96)); current-frame async quality/error-scope and final canvas handoff
qualification remain open, together with physical driver reset and OS input/AT.
The detached current-frame control patches are not applied, compiled or executed.
No runtime was rerun for this documentation. Later merged immutable-pin source
requires its own coherent supplier/lock checks.
