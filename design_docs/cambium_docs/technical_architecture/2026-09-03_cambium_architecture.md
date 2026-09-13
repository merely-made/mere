# Architecture

Cambium is an application toolkit over Genet. Meristem produces and reconciles
view structure. Cambium translates that structure into Genet's neutral DOM,
custom-leaf, presentation, and document-engine seams.

The dependency direction is one-way:

```text
applications -> Cambium -> Genet seams -> rendering and platform

Genet engine crates -X-> Cambium
```

## Ownership

- Meristem owns reactive diffing, messages, view identity, and view sequences.
- Cambium owns application views, controls, composition, and Genet adapters.
- Sprigging owns retained custom-leaf state and arrangement helpers.
- Genet owns DOM, style, layout, paint, input, accessibility, and browser
  behavior.
- Nematic and other document engines own parsing and protocol-faithful lowering.

Genet remains independently usable without Cambium. Sprigging is an extension
of Genet's neutral custom-leaf seam, not a second layout or input engine.

The published seam crates use Genet package names. Cambium's public backend
types use `Genet*` names; deprecated `Serval*` aliases are temporary source
compatibility shims.

## Surface lifetime (2026-09-08)

Application leaf identity and renderer-issued fragment identity have different
lifetimes. The native host preserves the application leaf registry, retained
layout and state during suspension, but retires `leaf_fragments` at the same
point it drops the renderer. Resume registers the retained leaves with its new
renderer, even when their content epochs have not changed.

S8 source review found that the old cleanup in `sync_leaf_fragments` could not
run through a suspended redraw: `redraw` returns before that call when there
is no surface. Clearing at `suspended` repairs that missed invalidation. Two
independent source reviews verified the path and the focused patch passes
`git diff --check`. Native suspend/resume GPU execution and rendered-pixel
acceptance remain open; this is a source correction, not that receipt.

## External viewport producers (2026-09-13)

Rootstock's `ProducerRegistry` binds an application renderer to an existing
`custom_leaf` key. The application owns scene data, depth, rendering and semantic
picking. Genet owns the content box, CSS composition, clipping and input geometry;
Rootstock owns registration, image staging and document lifetime. The specimen
bench consumer and its done-conditions live in
`isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md`,
Bench B. This seam does not introduce a second scene model or device.

The frame hook registers a `TextureProducer` through `AppCtx.producers`, declaring
the CSS properties its rendering depends on. After layout and before document
paint, `render` receives the host's device and queue, logical content extent,
physical extent including DPI and UI zoom, and a read-only `ResolvedAppearance`.
Declared `color` has a typed used value, including Genet's palette and color-scheme
resolution; RGB is encoded sRGB and alpha is straight. Other declared properties,
including custom properties, retain Genet's resolved CSS serialization. Border,
shadow and group opacity remain ordinary document composition unless explicitly
declared by the application.

A producer returns `None` when its inputs are unchanged, or a `ProducedTexture`
with view identity, generation, source alpha and source encoding. Rootstock stages
only changed output through Netrender's existing external-image route, then emits
`DrawExternalTexture` through the custom-leaf content slot. A changed view stages
even when the producer reuses its generation. The currently admitted encoding is
encoded sRGB sampled through an unorm view; linear-sRGB output is explicitly
refused because staging converts alpha but not color transfer functions.

Physical resize or device replacement sets `needs_frame`; an old-size image cannot remain
in the slot. A non-painted or zero-size leaf suspends its producer once and clears
the staged image. CSS visibility changes preserve registration. Removing a bound
DOM owner or the registration retires it; recreating an absent owner requires a
new registration. Native surface suspension clears staged images before dropping
the renderer. Invalid output and duplicate DOM keys produce an empty slot with a
queryable error, rather than presenting the previous frame. Producer keys share
Sprigging's namespace and must be registered in only one registry.
Retained GPU resources are bound to their device: after suspension, a producer
must compare the supplied device with its cached renderer or recreate that
renderer from retained application data.

Pointer delivery uses Genet's accumulated 2D paint transform and scroll mapping.
Custom-leaf events carry content-local coordinates; ordinary controls retain
border-local coordinates. Producer presses reject padding, clipping and singular
transforms, and overlaid DOM controls retain normal hit priority. Captured pointer
coordinates may leave the content box. The application maps admitted content
coordinates to its last rendered scene and owns any depth query or selection.

The API and lifecycle implementation are in
[`producer.rs`](../../../crates/cambium/cambium-rootstock/src/producer.rs) and
[`producer/registry.rs`](../../../crates/cambium/cambium-rootstock/src/producer/registry.rs).
`FrameProfile.producers` attributes producer rendering and image staging separately
from document raster and presentation. This is CPU attribution, not a GPU-time
measurement.

Validation on 2026-09-13 used Rust 1.97.1 and wgpu 30. Rootstock's 34 CPU tests
and two GPU tests passed, including ordinary custom-leaf paint-list translation,
document raster and byte-exact readback. The native host's seven integration
suites passed 62 tests covering input, zoom, focus, accessibility and lifecycle.
The producer fixture checks that decoration changes stage nothing, while typed
material changes, replacement, resize and recreation refresh the image. Linear
encoding and duplicate DOM keys refuse stale images.

A separate 598-package manifest checked the same host source and native test
files with `cargo +1.97.1 check --all-targets --release --offline`, using only the
committed engine patch entries and no ignored sibling redirects. Its lock resolves
21 Genet packages at `101d9e9ade8671564e723443d9f0498e899a33f1` and four Netrender
packages at `3961aca919f707ab09a786379eb4ce8bb121258e`, one source identity per family.
This is focused package validation; it does not claim a regenerated full Mere
workspace lock or a full workspace test run. Native specimen-bench acceptance
remains the consumer's receipt in the wing plan named above.
