# Cambium

Cambium is a Genet-native reactive GUI toolkit. It combines Meristem's
reactive view core with a Genet DOM backend and Sprigging custom leaves.

This repository was extracted from Genet's former Serval tree. Its public
backend vocabulary is now the `Genet*` family. Deprecated `Serval*` aliases
remain for source compatibility during consumer migration.

## Crates

- `meristem`: renderer-independent reactive diff and message core
- `cambium`: Genet backend, application runner, controls, and composition
- `cambium-genet-winit-host`: native lifecycle, reusable scene producers,
  caption controls and a windowless acceptance harness
- `cambium-winit`: winit keyboard translation for Cambium applications
- `sprigging`: engine-neutral custom leaves and arrangement geometry
- `mesquite`: the scenario lane over a document host — scenario ticking,
  captures, pixel checks, cost accounting and one JSON receipt
- [`tabard-workshop`](tabard-workshop): shared theme-authoring state, Cambium
  controls and preview scenes for embedding applications

Every crate is MPL-2.0 (see the repository `LICENSE`); Meristem, a Xilem
derivative, keeps the Xilem Authors' Apache-2.0 notice in each derived file.

See [the architecture doc](../../design_docs/cambium_docs/technical_architecture/2026-09-03_cambium_architecture.md)
for the ownership rule and
[the Xilem provenance ledger](../../design_docs/cambium_docs/technical_architecture/upstream-xilem.md)
for provenance. Licenses are recorded in the repository
[LICENSES.md](../../LICENSES.md), and the claimed package names in
[the namespace-claims doc](../../design_docs/cambium_docs/technical_architecture/namespace-claims.md).
Standalone and sibling-checkout development are described in
[the local Genet development doc](../../design_docs/cambium_docs/testing/local-genet-development.md).

## Shared appearance and title bars

`tabard` owns theme definitions, modes, derivation and interchange;
`tabard-workshop` provides their retained authoring surface. Tinct supplies
color math and Illume supplies syntax roles. Hosts keep application appearance
selection, document appearance precedence and authored-file destinations.

Cambium's `title_bar(ornament, title, actions, captions)` supplies four view
slots and `TITLE_BAR_CSS`. Mount that sheet, then override its CSS variables
or slot styles for the application's ornament. The title and ornament reserve
the native title-bar inset and form a drag region; interactive content belongs
in actions or captions, which exclude dragging. There is no second theme model.

For a native app frame, pass `platform_caption_controls(&commands, &labels)`
from `cambium-genet-winit-host` as the caption slot. macOS keeps its native
traffic lights; other platforms receive Minimize, Maximize and Close buttons.
Use `WindowFrame::App`, and set `HostOptions.maximize_control_label` to
`labels.maximize` so Windows Snap can find a localized Maximize control. Hosts
using `WindowFrame::Host` or embedding a panel should supply an empty caption
slot. Existing host behavior owns dragging, resize, double-click, the system
menu and close policy. The component composes their view; it does not implement
a second native window controller.

`SceneProducer<T>` adapts application scene/revision/accessibility callbacks to
the existing renderer and device. A view closure can capture native commands
without adding them to portable app state. `Harness::with_command_init` tests
that initialization against the host's actual command queue.

## Component acceptance surface

The executable component catalog covers Cambium's controls, hover routing,
editors, action list, overlay menu, virtualized grid, and Sprigging glyph
leaves. Run it with:

```sh
cargo run -p cambium --example component_catalog --all-features
```

The same assertions run in CI as an example test. See
[the component-catalog doc](../../design_docs/cambium_docs/technical_architecture/component-catalog.md)
for the coverage rule.

## License

MPL-2.0 (see the repository `LICENSE`). `meristem` retains the Xilem Authors'
Apache-2.0 notice as a derivative; see the repository
[`LICENSES.md`](../../LICENSES.md).
