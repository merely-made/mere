# Changelog

## Unreleased

- Remove `stylesheet_at_display_scale`. A native host composes at one layout
  scale, as the rootstock host does: sheets stay in layout pixels, layout runs
  at `physical / scale`, and the scene is rasterized at `scale`, so no host
  rewrites CSS lengths for a dense display.
- Draw the checkbox, toggle and radio indicators in the shell sheet, and give
  hosts what they were copying. Wrong before: the checkbox / toggle element held
  `[x]` / `[ ]` text and the radio put `(o) ` / `( ) ` inside its label's text
  node, so no stylesheet could replace them and hosts showed "( ) Genet" and
  "[ ]Hub room". Now the checkbox / toggle is an empty `control-mark` child inside
  the control, the radio row is a `radio-indicator` span beside a `radio-label`
  span, and `SHELL_CSS` draws all three from `currentColor`. `radio_options` takes
  `RadioOption`s (label, `disabled`, host attributes) with `aria-disabled`,
  arrow / Home / End movement over the enabled options, and a disabled saved
  selection that stays checked; `radio_group` is its label-only door.
  `checkbox_disabled` is an inert checkbox with `aria-disabled`. `status_note` /
  `status_note_labelled` give the `role="status"` note its base class and
  optional accessible name. `disclosure_trigger` is the disclosure button alone
  for hosts that keep the content elsewhere, and `RangeScrubber::with_value_text`
  names the value in words. Still open: the drawn radio dot and checkbox fill
  use `box-shadow` / `visibility`, so a renderer without them shows a solid
  indicator; a toggle has no disabled constructor yet.
- Publish the Cambium chain (Mark, 2026-10-19): `command-menu` 0.1.0 (the name
  is free on crates.io and stays; `alembic` is held by a third party), `mere-surface-api`
  0.1.0, `meristem` 0.2.0, `tinct` 0.2.0, `illume` 0.0.3, `sprigging` 0.2.2 and
  `cambium` 0.4.0. `cambium` takes a minor bump because the `nematic` feature
  removal below is breaking for the published 0.3.3. The published
  `netrender-vello` 0.11.0 sits on `netrender-vello-encoding` and
  `netrender-vello-shaders` 0.11.0, the vello fork's renamed `vello_encoding` and
  `vello_shaders` (library names unchanged), because upstream's 0.11.0 lacks the
  device-pixel curve quality bounds the fork's renderer uses.
- Rename the `edit-history` crate to `backstep` (`crates/cambium/backstep`, lib
  `backstep`; Mark, 2026-10-15). `cambium::History` is unchanged, re-exported
  from the renamed crate; only a direct `edit_history::` import changes, to
  `backstep::`. The crate stays standalone, has no dependencies, and Mark
  cleared it for publishing (`publish = false` is gone; `cargo publish
  --dry-run` passes). Cambium depends on it, so it has to be on crates.io
  before Cambium is.
- Let a graph canvas carry a consumer's node kinds, and split the detail
  popover's panel from its trigger, for widgets that live on graph nodes.
  `GraphCanvasSwatch::with_node_attr(id, name, value)` sets an attribute on
  one node's target (`aria-expanded`, `aria-haspopup`, a `data-*` kind); the
  canvas's own class, label, key and style are never overridden.
  `GraphCanvasSwatch::with_focus_request(id)` hands keyboard focus to one
  node's target when the request first appears, as a closed card or a
  dismissed popover returns it. `detail_popover_panel` renders a
  `detail_popover`'s peek or pinned panel for a trigger the caller already
  renders, such as a canvas node; `detail_popover` is built on it. mere-view's
  overmap tier is the first consumer. Additive.
- Remove the `nematic` feature and its `cambium::nematic` views
  (`gemtext_view`, `gopher_view`, `nex_view`, `feed_view`, `SmolwebTheme`,
  `stylesheet`), with the optional `errand` and `tabard` dependencies only it
  used. No consumer called them; smolweb renders through mere-document-lanes'
  EngineDocument lane (Mere stack seams plan, ruling S66; smolweb fidelity
  plan WS4, R0). Breaking for anyone who enabled the feature.
- Add `fold_projection`, a generic read-only source projection that validates a
  caller-held source-length witness, clips `StyleRange` highlights around
  collapsed regions, and renders `fold-marker` spans named "Folded content".
  It borrows caller-owned source and has no `TextInput` path; live editable
  folding still needs the coordinate map and hidden-input work.
- Freeze the retained surface contract at v1 with `genet-host-api`'s
  descriptor vocabulary. The descriptor keeps identity, label, and
  `accepted_source` (now the admission truth a host asserts against its
  provider registration); the speculative tail (roles, multiplicity,
  placement hint, potential capabilities) is removed and returns additively
  with a real consumer. `RetainedSurfaceSession` keeps all fourteen
  methods. Changes are additive until a v2.
- Add an application scroll request to `cambium-rootstock`:
  `AppCtx::scroll_into_view(node, ScrollAlign)` queues a request that the host
  resolves on its next layout against the node's painted rect. The nearest
  plane that can move (an ancestor that scrolls vertically and has room to,
  otherwise the window viewport) moves by the alignment. Each plane outside
  it, out to the window, then moves only as far as the element needs, so an
  element in a scroller that is itself out of sight comes into view. Every
  plane is clamped to its range. `ScrollAlign::Start` puts the element's top
  at the top of that nearest scrollport, inside the container's border;
  `Nearest` moves only as far as a partly hidden element needs. A moved
  caret is kept in view down the same planes. A node that is gone or does
  not paint is a no-op. Both event sources honour it through `Host::relayout`, and the
  windowless `Harness` in `cambium-genet-winit-host` gains `viewport_scroll`.
  Both crates are unpublished, so no version moves.

## 0.3.3 - 2026-08-09

- Publish the post-0.3.2 workspace state under its own number. The registry
  0.3.2 lacked the additions below, which made the local tree and the
  published crate diverge under one version and failed `cambium-winit`'s
  package verification against the registry.
- Add the shared text-editing primitive and the IME composition surface
  (`CompositionEvent`, `Key::Composition`) that `cambium-winit` 0.3.0
  translates into.
- Move radio-group behavior into Cambium and repair the component-catalog
  receipt gate.
- Canonicalize organization repository URLs.

## Unreleased

- Version `meristem` as 0.2.0 for the public scope cut: remove the unused
  worker, task, environment, and documentation-backend APIs, and keep the
  retained diff and message core.
- Split the Genet/AccessKit accessibility host out of `cambium-winit` into
  the deliberately unpublishable `cambium-winit-a11y` (2026-07-26), leaving
  `cambium-winit` at `cambium` + `winit` only and publishable again. The
  0.3.0 note below ("crates.io publication waits for the standalone
  `genet-layout` package boundary") is superseded by that split.
- Release-record reconciliation (2026-08-09): `cambium` 0.3.3 and
  `cambium-winit` 0.3.0 published the same day, closing the registry gap;
  `meristem` 0.1.1, `sprigging` 0.2.1, and `cambium-nematic` 0.3.1 were
  current at that reconciliation; `cambium-winit` 0.2.0 remains yanked.
  Authority:
  `design_docs/cambium_docs/technical_architecture/genet-compatibility.md`.

## 0.3.0 - 2026-07-22

- Add `HoverEvent`, `HoverPhase`, `on_hover`, and runner dispatch seams for
  host-computed Enter, Leave, and Move transitions.
- Expand the component catalog into an executable acceptance surface covering
  controls, editors, action routing, overlays, grid virtualization, semantic
  attributes, keyboard behavior, and Sprigging leaf painting.
- Give data grids explicit grid, row, column-header, and cell semantics, with
  keyboard activation for sortable headers.
- Use the canonical `genet_scripted_dom` Rust crate name throughout Cambium,
  Cambium Nematic, tests, and examples.
- Replace stale Serval-era crate documentation with the current Cambium,
  Meristem, Sprigging, and Genet ownership boundary.
- Add the retained `GraphCanvas` leaf plus Cambium's interactive
  `graph_canvas_swatch`, including focus, data keys, optional expansion, and
  visible node labels.
- Add shared overlay, command-menu, disclosure, summary, detail, sectioned-list,
  split, tab, segmented-control, and reorderable-list surfaces.
- Add `caret_text_field` for DOM-painted carets and forest-DOM projections for
  multiple window roots over one application state.
- Add Cambium Winit scroll policy and the Genet/AccessKit accessibility host.
  The source release is versioned here; crates.io publication waits for the
  standalone `genet-layout` package boundary.

## 0.2.0 - 2026-07-14

- Make `GenetCtx`, `GenetElement`, `GenetAppRunner`, and related `Genet*`
  names canonical. Deprecated `Serval*` aliases remain for migration.
- Make buttons, checkboxes, switches, radio groups, selects, and sliders follow
  standard keyboard and accessibility interaction patterns.
- Add the searchable, keyboard-complete `action_list` component.
- Make normal manifests resolve Genet seams from crates.io so a standalone
  checkout does not require a sibling Genet repository.
- Add CI for formatting, focused Clippy, workspace tests, and package checks.
