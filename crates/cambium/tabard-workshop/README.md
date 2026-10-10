# tabard-workshop

Shared appearance-authoring state and a Cambium surface for embedding
applications. Turnstone, Woodshed, Knot and Cleromancy can depend on this
package using a `tabard-workshop` Git dependency on the Mere revision they
adopt (the Mere workspace exposes the same name). They can mount the
same authoring workflow used by the standalone Tabard desktop.

The theme definition, registry, draft, portable artifact and library APIs live
in [`tabard`](../../system/tabard). This package composes those APIs with
Cambium controls, syntax highlighting, the graph swatch and document preview
scenes. It owns one retained `WorkshopState`; an embedding host supplies its
existing window, event routing and renderer. The package has no runtime
dependency on the desktop port.

The [Tabard product guide](../../../ports/tabard/README.md) describes controls,
import/export behavior, persistence guarantees and current capabilities. The
[desktop host](../../../ports/tabard/desktop) is the native reference adapter;
native scenarios, frame captures and receipts remain with that product port.

## Mounting the workshop

Create `WorkshopState::in_memory()` for a session-only library, or
`WorkshopState::load(path)` for an explicitly chosen authored-theme library.
Mount `workshop_view` with `workshop_stylesheet()`. The stylesheet composes
Cambium's shared title-bar, syntax and graph rules with the workshop frame.

For a desktop window, `workshop_view_with_captions(state, captions)` accepts
host-supplied window controls. The default `workshop_view` supplies an empty
caption slot for embedded panels. The header uses Cambium's `title_bar` and
`TITLE_BAR_CSS`; the host owns frame policy and window commands. Native hosts
can use `platform_caption_controls` with their existing `WindowCommands`.

The workshop edits an isolated user draft. Preview mode is local to the editor;
it does not change another application's active appearance. The embedding
host chooses when and how to apply an authored theme using Tabard's shared
definition and mode APIs. Authored library saves and editor-selection settings
use the same implementations as the standalone product.

Register `graph_leaf()` under `GRAPH_LEAF_KEY`. `reader_preview()` and
`stylesheet_preview()` return retained scene handles implementing `PreviewScene`
(`frame`, `revision` and `accessible_name`). Their host texture producers use
`READER_LEAF_KEY` and `STYLESHEET_LEAF_KEY`. The native host's shared
`SceneProducer::new(source, raster_key, frame, revision, semantics)` adapts these
callbacks to its existing `TextureProducer` lifecycle. Give independent scenes
distinct raster keys, increment revisions when visible content changes, and
provide the current accessibility projection. Register these producers with the
host's existing render resources and rebind the handles when reopening replaces
the workshop state. The workspace's native adapter demonstrates those seams
without allocating a second editor model or renderer.

## Host events and file operations

Ordinary control messages synchronize the authoring model. Native text and IME
integration supplies focused slots through `text_field` and `text_field_mut`
for `name`, `seed-hex` and `mode-sheet`, then calls `sync_controls()` after
dispatch.

Supply Cambium's existing file-chooser seam for imports. For exports, consume
`take_export()`, choose a destination in the host, and call `complete_export()`.
The shared model validates, writes and handles any replacement confirmation;
the host does not duplicate file-publication or draft logic.

Route close requests through `request_close()` and honor `exit_requested()`.
The workshop keeps staged or unsaved work available when validation or saving
fails. Native window and application-close policy belongs to the host.

## Shared native binding

Enable the optional `native-host` feature to use `native_host::WorkshopHost`,
`native_init` and `native_hooks` for a workshop window. Pass the application's
existing render core to the ordinary Winit host constructor and route that
window's events from the application's event loop. The binding supplies
preview registration, native caption controls, controlled text, explicit
export effects and the shared dirty-close decision.

For an embedded workshop within another application state, retain one
`PreviewBindings` value and call `register` with the current workshop and the
parent's leaf/producer registries. Call `sync_and_export` after dispatch and
use `focused_field` to project the parent's controlled-text slot. Independent
bindings allocate distinct retained raster identities on the same core.
Application selection, persistence, window routing and final exit policy stay
with the parent. The feature and its native dependencies are excluded on Wasm.

The native binding's complete CPU gate is:

```sh
cargo test --locked --offline -p tabard-workshop --features native-host --lib
```

## Fixtures and checks

The reader source and isolated application document are checked-in
[fixtures](fixtures) beside the shared package. Relative source includes remain
local to the package, and the retained acceptance tests move with it. Run:

```sh
cargo test -p tabard-workshop
```

Native lifecycle tests and GPU capture scenarios are documented by the
[desktop host](../../../ports/tabard/desktop/README.md).

## License

MPL-2.0 (see the repository [LICENSE](../../../LICENSE)).
