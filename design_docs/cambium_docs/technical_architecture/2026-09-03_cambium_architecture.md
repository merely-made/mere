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

## Scroll planes across a rebuild (2026-09-15)

Rootstock's retained layout owns two scroll planes: the document's viewport offset
and a per-node map of offsets for nested `overflow: auto` containers. Both are carried
across a layout rebuild, and both are now clamped against the layout that rebuild
produced. Nested entries are additionally pruned: a node that no longer resolves a
scrolling overflow — because its style changed, or because it left the DOM — drops its
offset instead of holding a position no box can occupy. Previously only the viewport
plane was re-clamped, so an application whose nested box shrank kept a stale offset.

The clamp runs at the end of `layout_resolved` for an in-place rebuild, and again inside
`set_element_scroll` for the branch that constructs a fresh session and carries the plane
onto it — that plane arrives after the new session has already laid out, so a clamp in
`layout_resolved` alone would never see it. The shape mirrors genet-livery's
`clamp_nested_scroll` (`genet/components/genet-livery/src/document/scrolling.rs`), but
rootstock reuses its own `element_scroll_range` and its own overflow test rather than
copying code across repositories. Implementation in
[`owned_layout.rs`](../../../crates/cambium/cambium-rootstock/src/owned_layout.rs), with
four cases in
[`owned_layout/tests.rs`](../../../crates/cambium/cambium-rootstock/src/owned_layout/tests.rs):
a shrinking container, a box that stops scrolling or leaves the DOM, an untouched
sibling, and a plane carried onto a fresh session. Three of the four fail with the clamp
calls removed. Validated on Rust 1.97.1, `--offline --locked`: rootstock's 40 tests and
the native host's 80 across nine suites.

## Scroll requests (2026-09-16)

An application can ask the host to bring one of its own elements into view. The two
scroll planes above stay host-owned and their setters stay crate-private; what an
application gets is a request. Decision 18 of the Micron navigation plan
(`nematic_docs/implementation_strategy/2026-09-15_micron_navigation_plan.md`) settled
that the request belongs to Cambium, for every Cambium application, rather than Knot's
preview becoming a scroll container of its own.

```rust
// Illustrative: an `after_dispatch` hook revealing a node it found in the DOM.
ctx.scroll_into_view(node, ScrollAlign::Start);
```

**Where the request is made.** Cambium's view handlers see only application state; an
application reaches the host through its hooks, and `AppCtx` already carries requests
the host applies once a hook returns: `set_sheet`, `set_ui_zoom`, `pointer`, `capture`.
`AppCtx::scroll_into_view(node, align)` joins them, available from every hook. It
pushes a `ScrollIntoView` onto `HostState::pending_scroll`. It is not resolved when the
hook returns, because the dispatch that asked may have created the node and the next
layout has not seen it yet, and a `frame` hook runs before the first layout exists.
`Host::relayout` resolves the queue in order once its layout is current, whichever
branch brought it there (tick, in-place rebuild, or a fresh session carrying both
planes). Each request resolves on its own, so one that finds nothing drops only itself.
A hook running after the frame (`after_frame`) would otherwise wait for an unrelated
redraw, so applying a hook's requests also asks the window for one. A plane that moves
notes the overlay-scrollbar fade, as the wheel default does.

**How the element is named.** By `NodeId`, the identity `AppCtx::painted_rect` already
takes, and the one `runner.set_focus`, `runner.focusables()`, `FocusedTextSlot` and
`A11yRequest` use; the harness's `taproot` selectors resolve to it too. An application
that can ask where an element paints can ask for the same element to be shown. Genet's
node ids are monotonic and never reused, so a request that outlives its element misses
rather than landing on another one. Three alternatives were weighed. A `taproot`
selector would make rootstock depend on the probe crate, and a role or label is not
identity. A DOM `id` string would require authored ids and a document walk per request.
A view wrapper in the shape of `request_focus` would name the element structurally, but
would put a layout-dependent request into `GenetAppRunner`, which stays free of layout
and presentation. Such a wrapper could later be sugar over this request.

**Which plane moves.** Exactly one: the nearest ancestor whose computed `overflow-y`
scrolls *and* whose vertical range is positive, otherwise the window viewport. The range
condition is load-bearing. Knot's `.knot-scroll-preview` and `.knot-workspace` are both
`overflow: auto` but grow to their content, so the window carries the offset; a rule
that stopped at the first scrolling overflow would pick a box with no room and move
nothing. The wheel default passes over such a box for the same reason. Unlike the DOM's
`scrollIntoView`, outer planes are not chained: a container that is itself off screen is
scrolled but not brought into the window. The request is vertical only and leaves
horizontal offsets alone. There is no animation.

**Alignment.** `ScrollAlign::Start` puts the element's top edge at the top of the scroll
area: the viewport's top, or the container's box as `element_scroll_range` measures it.
`ScrollAlign::Nearest` does nothing for a fully visible element; otherwise it moves the
top edge to the area's top when the element sits above, and the bottom edge to the
area's bottom when below, with an element taller than the area aligning its top. It was
a few lines once `Start` existed. Both clamp to the plane's range, and the existing
rebuild clamp keeps a carried offset inside shorter content later.

**Hosts.** Both event sources honour it with no code of their own. The winit source
derefs to rootstock's `Host`, and the browser source calls the same `Host::redraw` and,
on resize, `Host::relayout`. `cambium-genet-web-host` was checked for
`wasm32-unknown-unknown` against this change. The windowless `Harness` gained
`viewport_scroll()` so a test can tell the window's offset from a container's.

Implementation: `AppCtx::scroll_into_view` and `ScrollIntoView` in
[`host.rs`](../../../crates/cambium/cambium-rootstock/src/host.rs), resolution in
`Host::relayout` in [`frame.rs`](../../../crates/cambium/cambium-rootstock/src/frame.rs),
and `ScrollAlign` with `OwnedLayout::scroll_into_view` in
[`owned_layout.rs`](../../../crates/cambium/cambium-rootstock/src/owned_layout.rs).
Eight harness cases in
[`tests/scroll_request.rs`](../../../crates/cambium/cambium-genet-winit-host/tests/scroll_request.rs)
drive the request from an `after_dispatch` hook: an element far below the fold reaches
the viewport top (and an element above the new offset comes back up); a nested
container scrolls and the window does not; a grow-to-content `overflow: auto` box is
passed over for the window; a stale node is a no-op and does not stop the requests
queued with it; a request made before the first layout resolves after it; both planes
survive an in-place rebuild and a fresh session; an offset past the content's end
clamps at the request and again when the content shrinks; and `Nearest` moves only as
far as needed. Eleven positive controls each broke one rule and watched the named test
fail. Validated on Rust 1.97.1, `--offline --locked`: rootstock's 40 tests, the native
host's 88 across ten suites, cambium's 216, mere-document-lanes with `smolweb` 33, the
web host checked natively and for wasm32, the format check on the three host crates,
and the workspace check.
