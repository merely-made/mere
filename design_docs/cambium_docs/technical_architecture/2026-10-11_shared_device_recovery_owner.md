# ADR: One recovery owner for a shared render device

**Status:** Implemented; native controlled-loss/current-frame and browser controlled shared replacement R6 passed. Browser current-frame scopes/counters remain open.
**Date:** 2026-10-11
**Deciders:** Renderer implementation owner under the user-authorized recovery work.

## Context

Several native windows share one RenderCore, while each Rootstock host retains
its own runner, layout, producers, captures and presentation surface. A lost
device invalidates all of those GPU bindings together. Surface loss is distinct
from device loss. Earlier exact two-device recreation was an orderly teardown,
not recovery from the installed device-loss callback.

## Decision

Genet owns one SharedRenderCore per ownership group, with a stored replacement
recipe and immutable generations. Attaching the same core reuses its existing
owner rather than creating a second per-window factory. One device callback
records a typed notice; subscriptions fan out wake notifications without
executing callbacks under owner locks.

Loss starts a detachment barrier. Every attached host drops its old surface,
producer images, GPU fragment bindings and pending readback/capture ownership,
then acknowledges that generation. Closed subscriptions leave the barrier.
Only after all live hosts acknowledge may one replacement factory run. The
factory executes outside the owner lock, and its failure remains terminal for
that transition. Repeated or stale notices cannot cause extra boot attempts.
A later distinct generation may start a new transition.

Rootstock handles the notice before raster/producer work and calls its actual
producer registry suspension path. It preserves application-owned CPU sources,
producer registration, runner, authored draft, appearance, focus and layout.
Typed capture/readback refusals distinguish lost-device frames from success;
armed requests are resolved once. Successful hosts create fresh surfaces on
the replacement core. No host may render using a detached old generation.

Outdated surfaces reconfigure on the healthy current device. A lost surface
requests recreation of that window's surface and does not trigger shared
replacement by itself. Surface recreation failure stays unavailable.

## Options considered

1. Per-window replacement: simple locally, but can boot multiple devices and
   leave other windows using old resource generations. Rejected.
2. Immediate shared replacement on first notice: one factory, but can race
   outstanding host surfaces, producer bindings and captures. Rejected.
3. Shared owner plus detachment barrier: explicit group lifecycle and one
   bounded replacement attempt; selected.

## Consequences

Capture callback contracts gain typed Result/refusal outcomes, requiring narrow
consumer ports. Hosts sharing a core must participate in the same lifecycle.
GPU-only producers can restore only according to their existing source-owner
contract; the recovery owner does not invent missing portable resources.
Independent host registries remap local producer keys to distinct stable physical
renderer identities. Shared-forest lifecycle/disposal remains a separate contract.

Native blocking replacement uses the retained boot recipe. Browser replacement
uses the stored asynchronous recipe and one owner/generation/attempt-branded
claim after the same detach barrier. The adapter launches its local non-Send
future outside host borrows. A monotonic deadline prevents late Ready admission,
including delayed JS timers; expiry/drop/failure is sticky for that transition.
Synchronous polling leaves asynchronous replacement pending. Controlled
Device::destroy validation is distinct from a physical driver reset.

## Validation gates

- Actual monitor stale/dedup notices and two-host acknowledgement/factory-failure controls.
- Actual Rootstock registry, source-authority and exact-once capture-refusal controls.
- Installed callback after controlled device destruction, one fresh shared generation,
  registered producer pixel rehydration, and a later bounded failed replacement.
- Native surface acquisition/recreation checks and web compilation of the callback ABI.
- Source/pin identities and unchanged pixel acceptance thresholds in durable receipts.

Executed qualification is recorded in the [production flattening and device recovery receipt](../testing/receipts/2026-10-12_flattening_and_device_recovery/README.md). The actual installed callback after controlled device destruction, one shared replacement, two registered reader pixel rehydrations, exact-once refusal and bounded later factory failure passed. Current native shader counters are observed before raster success, including an actual late round-cap limit refusal. Healthy native presentation through that completion fence passed the unchanged scenario after console unlock: 53 presented frames and six nonblank, distinct captures. A separate production one-window control now qualifies new-surface queue presentation after actual owned-device destruction: 56 presented frames and four nonblank captures, preserved authority/source identities and fresh preview generations. Pixels pass unchanged existing limits (max RGB 1, alpha 0); this is not physical driver reset, multiple native windows or a complete hidden-producer pixel comparison. The Wasm callback ABI is recorded separately. The later [browser R6 record](../testing/receipts/2026-10-12_flattening_and_device_recovery/browser-r6/README.md) qualifies the actual asynchronous owner with two full Workshop mounts, one application-hidden host, same-canvas new generation and all complete registered producer images. All twelve PNGs and six whole-image comparisons pass unchanged RGB 3 / alpha 2 / mean 1 limits, with maximum observed RGB 1 and alpha 0. Explicit outer/producer font ledgers use the fixture's Roboto-to-four-generics delivery choice; production typography fidelity is separate. The [two-window native R2 record](../testing/receipts/2026-10-12_flattening_and_device_recovery/native-shared-r2/README.md) retains its earlier G09b/Mcc65 source bounds and does not requalify the later font changes. Browser current-frame error scopes/counters and final canvas clear/blit remain open, as do physical driver reset, OS AT and comprehensive producer-upload/composition loss scoping.
