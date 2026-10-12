# Shared render-core device-loss recovery

The application owns one `SharedRenderCore` for every group of hosts using the
same device. A `RenderCore` has an immutable generation and one wgpu loss
callback. Attaching another host to the same core reuses its existing owner;
the first owner's replacement factory remains authoritative. The factory
captures the original boot recipe, including backend and feature policy.

The callback records one generation-branded notice, cancels owned readbacks,
then wakes every subscribed host. Callbacks execute outside monitor/owner
locks. Readback cancellation drops its buffer, map receiver and device handles
before recovery begins, even if its consumer has not polled. The terminal
result is `ReadbackFailure::DeviceLost` and is delivered once. An explicit
second collection returns the distinct `AlreadyCollected` misuse refusal;
it cannot replay the loss notice or pixels.

Each host polls before GPU work. On a current loss, it removes its old surface
and presentation identity, clears GPU retained-fragment bindings, suspends the
actual shared producer registry, and detaches its `AppShared::render_core`.
The registry retains registrations and portable CPU sources. The runner,
retained layout, text selection, authored drafts, stylesheet and CPU leaf
commands remain authoritative. Producers receive the replacement core and
its explicit `generation()`; registry device tracking checks that generation
as well as the device handle.

Recovery waits until every subscribed host acknowledges detachment. A closed
host drops its subscription, removes itself from the barrier and wakes the
remaining hosts. Stale generations and subscriptions from another ownership
group cannot acknowledge a transition. The shared factory runs once, outside
the owner lock, after the barrier opens. A factory error produces a sticky
typed `Failed` state, rather than an implicit retry loop. A successful new
core must have a different generation and belong to this owner. All hosts
adopt that same replacement core and configure their own surfaces without
rerunning application initialization.

A configured replacement core is not sufficient to claim a presented frame.
Winit configures a new surface against its existing native window. A surface
creation failure is `SurfaceUnavailable`, retains application state and
leaves the old surface detached. Surface `Lost` on a healthy device recreates
only that window's surface on the same core; `Outdated` reconfigures that
surface. Neither event initiates device replacement. Successful presentation
remains the existing queue-presentation boundary.

## Capture API decision

The former callbacks accepted only successful surface/view arguments. They
could not report a lost device and callers would wait for a timeout after the
host discarded their callbacks. `CaptureFn`, `StampedCaptureFn` and
`PaintCaptureFn` now receive `Result<Frame, CaptureRefusal>`. The host takes each
armed callback once and delivers `DeviceLost(old_generation)` when detaching.
Surface configuration failures deliver `SurfaceUnavailable`. Mesquite's
native and browser backends propagate refusals through the original pending
request; paint sidecars also preserve explicit failure. This is an intentional
public callback ABI change, ported through the workspace's native/browser
consumers and receipt examples.

## Checked raster and producer refusals

Host rasterization uses Netrender's checked rendering API, including filtered
scene prefixes. Provider errors return normally through Genet's
`RasterizeFailure::ProviderError`, preserving the actual typed Vello error.
The installed loss monitor is checked after raster admission/submission: a
recorded device loss takes precedence and becomes `DeviceLost`. Raster creation
holds an admission lifetime so replacement cannot begin while it still owns
resources on the old generation.

`TextureProducer::render_checked` defaults to the existing producer contract;
`SceneProducer` overrides it to report the checked raster result. The registry
invalidates a refused source's previous image and transient texture, records
its source key, and stops the current host frame. It retains the registration
and application-owned portable source. A healthy provider refusal must never
be interpreted as unchanged output, missing content, or device loss.

Rootstock resolves armed captures with `CaptureRefusal::ProviderError` on a
healthy provider refusal. It clears that frame's presentation identity and
keeps the same recovery owner/device; no replacement factory runs. Actual
device loss uses the existing detach/barrier recovery path. The capture
refusal and raster failure types preserve provider errors through `Arc`;
provider errors are not equality-comparable, so `CaptureRefusal` no longer
implements `PartialEq`/`Eq`.

## Validation scope

The coordinator runs pure controls through the production monitor, owner,
host handler and deliberately failing replacement factory. These prove
deduplication, detach-barrier behavior, stale/foreign acknowledgements,
exactly-once capture refusal and retained application/source authority. They
do not claim that a GPU device was created or destroyed.

A separately ignored, bounded integration test destroys an explicitly owned
wgpu device, awaits its real loss callback, invokes the same host handler,
recreates real resources and compares actual producer pixels. Its receipts
declare controlled device destruction, not forced driver reset, and distinguish
headless resource/readback evidence from native presentation. Independent
producer registries sharing a renderer still require distinct external image
keys; the test does not hide that existing namespace constraint.

Browser readbacks share cancellation and typed refusal. The default synchronous
replacement recipe cannot request a new WebGPU device on the browser event
loop and therefore reports explicit failure there. A browser asynchronous
replacement owner is a separate extension; native success does not qualify it.

Native checked raster targets now require current Classic shader-counter completion before success. Genet observes zero pending counter frames/bytes inside loss admission and API scopes, with a five-second deadline enforced before polling, before success and after scope observation. Current shader-limit failures refuse the frame before producer image staging, presentation or capture; CPU producer registration survives refusal. Browser asynchronous counter/error-scope qualification remains open; this native blocking completion cost is separate from render encode/submit timing.

Executed controls and source/pin identities are in the [qualification receipt](../../../design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md). Controlled destruction/rehydration and healthy native surface presentation are separate receipts; physical driver reset and browser async recovery are not claimed.

The production one-window Tabard acceptance control now observes the actual installed Destroyed callback and a fresh shared generation/surface, preserving authored state and both named preview source identities. It records per-image generation/stamp/size/owner without retaining a GPU handle, ties observations to successful presentations, and compares displayed RGBA under the existing stricter interior/mean limits applied uniformly. Source-reviewed loss branches explicitly drop their frame-local raster and swapchain handles before polling replacement. The runtime injection is after a completed baseline capture; arbitrary concurrent upload/composition loss, multiple native windows, OS accessibility, browser async recovery and physical driver reset remain separate gates.
