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

The frame uses Cambium's opt-in `INSTRUMENT_CSS` and
`workspace_pane_header`: continuous pane edges, compact utility controls,
source/context captions, and separate active, selected, unsaved and loading
states. The existing split/tab and title-bar components keep their geometry,
input and window-policy ownership. Hosts mark utility-control regions with `workspace-controls`; title-bar and pane
actions are already admitted regions. Content buttons, including graph semantic
hit targets, keep their own presentation and geometry. Theme specimens retain
their own draft colors and typography.

`workshop_stylesheet()` supplies a neutral built-in light frame through
Tabard's existing palette derivation. An embedding host can instead use
`workshop_stylesheet_with_frame(applied_appearance_stylesheet)` to supply its
applied appearance. The frame is independent of the edited definition; this
operation neither saves the draft nor applies it to the embedding app.
Font-limited hosts supply `--workspace-font-interface` with their admitted
default family. The shared skin retains it after authored family requests;
it does not assume that a browser host has a system font collection.
The workshop's `data-theme-definition` remains the actual portable theme ID
through mode and layout changes.

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
`READER_LEAF_KEY` and `STYLESHEET_LEAF_KEY`. Rootstock's shared
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
for `name`, `seed-hex`, `mode-sheet`, `font-family`, `font-size` and
`font-line-height`, then calls `sync_controls()` after
dispatch.

Supply Cambium's existing file-chooser seam for imports. For exports, consume
`take_export()`, choose a destination in the host, and call `complete_export()`.
The shared model validates, writes and handles any replacement confirmation;
the host does not duplicate file-publication or draft logic.

Route close requests through `request_close()` and honor `exit_requested()`.
The workshop keeps staged or unsaved work available when validation or saving
fails. Native window and application-close policy belongs to the host.

## Shared preview and native binding

Enable the optional `native-host` feature to use `native_host::WorkshopHost`,
`native_init` and `native_hooks` for a workshop window. Pass the application's
existing render core to the ordinary Winit host constructor and route that
window's events from the application's event loop. The binding supplies
preview registration, native caption controls, controlled text, explicit
export effects and the shared dirty-close decision.

Enable `host-previews` to use the same `PreviewBindings` on native or browser
hosts. For an embedded workshop within another application state, retain one
`PreviewBindings` value and call `register` with the current workshop and the
parent's leaf/producer registries. Call `sync_and_export` after dispatch and
use `focused_field` to project the parent's controlled-text slot. Independent
bindings allocate distinct retained raster identities on the same core.
Application selection, persistence, window routing and final exit policy stay
with the parent. Only the `native-host` adapter and its native dependencies are excluded on Wasm.

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

## Typography

The shared workshop edits Tabard's five optional font roles: Interface, Reading,
Headings, Labels and Code. Blank fields preserve the consumer's defaults. Sizes
use CSS pixels; line height is a multiplier. Apply font preferences updates the
isolated previews, while Save persists the definition and the application owns
its separate Apply action. Invalid input stays visible and blocks publication
and navigation. Undo, Redo, Discard and per-role reset use the existing draft.

Supply `set_installed_font_families` from the host's existing font discovery
owner. The native binding uses Genet Parley/Fontique through the shared Cambium
host helper. Availability is unknown when a host supplies no catalogue; an
unavailable requested family is retained with a visible generic fallback notice.
Families are searchable suggestions rather than downloaded or embedded faces.

The application stylesheet specimen maps roles through ordinary CSS. The reader
specimen uses `mere_document_lanes::document_style_with_typography` and the
existing document-canvas style/measurement path, preserving heading hierarchy
and unrelated role defaults. The author CSS follows preferences in the cascade.
The editor's own neutral text remains independent of the authored theme.

The visible application chrome and syntax samples consume the same Tabard role
declarations as the isolated stylesheet and production adapters. Interface fonts
reach sample toolbar/actions, Heading reaches the sample title, Label reaches
address/filename text, and Code reaches the Illume-highlighted runs. Font variables
stay within each sample; Cambium appends font declarations while retaining the
local syntax palette. The authoring controls keep their neutral font styles.

## Retained host font resources

Supply the outer Host's admitted faces through `Init::fonts`. `HostFont` with
`sans-serif`, `serif`, `monospace` or `system-ui` binds the actual CSS generic
and retains the prior quoted named-family alias. The sheet and authored theme
keep their family requests; resource registration does not rewrite them.
Font selection remains host policy. Choose actual covering faces appropriate to
the role, rather than assuming a browser has a native system font collection.

The Reader and Stylesheet previews each own an independent retained document.
Outer Host fonts do not automatically reach those text systems. Before their
first frame, supply the same appropriate explicit resources through
`ReaderSpecimen::set_generic_fonts` and
`StylesheetSpecimen::set_generic_fonts`. Each accepts `(generic, bytes)` entries,
returns a typed refusal on invalid input, and changes its source revision only
when an admitted ledger changes. A failed replacement preserves the old ledger
and revision. Palette, author stylesheet and typography reconstruction retain
those bindings; viewport reflow uses the same resources. Reopening may replace
the source handle, so the embedding application supplies resources to that new
source and lets its existing `PreviewBindings` rebind it.

The production source and document layout still own glyph shaping, scene
commands and semantic text. The application owns draft, library, selection and
appearance application. No separate preview DOM or second application model is
needed. The [retained font control](tests/generic_fonts.rs) checks selected face
bytes through actual Reader/Stylesheet frames after profile, typography and
viewport changes, including failed replacement. The browser delivery fixture's
Roboto-to-four-generics choice qualifies transport only; it does not qualify
serif, fixed-width or locale coverage for production consumers.
