# ADR: One recovery owner for a shared render device

**Status:** Accepted for implementation; qualification pending.
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
Existing renderer-global producer-key namespace constraints remain explicit.

Native blocking replacement uses the retained boot recipe. Browser acquisition
is asynchronous, so any browser-specific factory limitation must be recorded as
a typed refusal rather than treated as a successful replacement. Controlled
Device::destroy validation is distinct from a physical driver reset.

## Validation gates

- Actual monitor stale/dedup notices and two-host acknowledgement/factory-failure controls.
- Actual Rootstock registry, source-authority and exact-once capture-refusal controls.
- Installed callback after controlled device destruction, one fresh shared generation,
  registered producer pixel rehydration, and a later bounded failed replacement.
- Native surface acquisition/recreation checks and web compilation of the callback ABI.
- Source/pin identities and unchanged pixel acceptance thresholds in durable receipts.

No successful runtime recovery is claimed by this decision record. Qualification
results must be linked after execution.
