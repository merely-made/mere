# Chart, Tab and Score in Graphshell — 2026-10-19

This packet qualifies a bounded implementation in the canonical Mere and
Woodshed **main working checkouts**. It is local, uncommitted source integration;
no commit, push, deployment or portable cross-repository Git pin is claimed.

Graphshell reads the progression block's Set-owned MusicXML through
`woodshedding`. If no MusicXML exists, the explicit **Write notation** action
uses the shared selected-shape resolver and conversion rules. It completes
validation before patching only the music field in the retained JSON tree,
preserves unknown fields and source body, and participates in the existing
Knot document undo/save path. Reading/view/attention changes do not author.

The shared `woodshed-notation` crate owns the existing Sprigging leaf, layout,
Bravura glyph outlines and license, label positions, and event rectangles.
Both host adapters supply colors and own their controls and persistence.
The canonical native Arrangement panel can read the same Set music through
that leaf. The native host was checked and its reading tested, without a new
native window, device or audio run.

## Evidence

- `receipt.json`: decisions, per-lane test counts, browser results and limits.
- `source.json`: 48 selected source/manifest/asset hashes and both repository
  base heads. Shared manifests include the pre-existing Woodshed audio work;
  that unrelated work was preserved, not claimed as part of this change.
- `package.json`: exact generated browser package hashes. The large development
  package stays outside Git under `Code/output/graphshell-notation-20261019/pkg`.
- `checks/`: 201 model tests and 4 doctests, 24 Graphshell document tests,
  11 selected-shape regressions, 4 conversion cases, 8 renderer cases,
  5 source-parser cases, and 2 native reading/metadata preservation cases;
  canonical browser product build, native host check and port boundaries.
- `browser/notation_readings/`: real IAB WebGPU narrow workflow, including
  explicit notation creation, Chart/Tab/Score selection, carried rest attention,
  source preservation, one-step undo/redo, and acknowledged document save.
- `browser/notation_readings_reopen/`: new-page wide reopening of that saved
  document and unchanged-source Score/Tab readings.
- Five reviewed render captures cover 455×1220 and 1280×900 CSS pixels at DPR 2.
- `packet-manifest.json`: hashes of packet members.

## Reproduction

From Mere:

```sh
cargo test --locked --offline -p graphshell --no-default-features --features editable-blocks block_editing --lib
cargo test --locked --offline -p illume blocks::tests
python3 scripts/check_port_boundaries.py
```

From Woodshed:

```sh
cargo test --locked --offline -p woodshedding
cargo test --locked --offline -p woodshed-core --lib card_shapes
cargo test --locked --offline -p woodshed-core --lib music::
cargo test --locked --offline -p woodshed-notation --lib
cargo test --locked --offline -p woodshed-views --lib notation_reading
cargo test --locked --offline -p woodshed-views --lib ordinary_arrangement_draft_preserves_authored_meter_and_groups
cargo check --locked --offline -p woodshed-genet
```

The portable model run recorded here used an isolated manifest pointing directly
to the canonical `woodshedding/src/lib.rs` (the complete 201-case unit suite and
four crate-named doctests), avoiding unrelated workspace dependency resolution.
Other checks above used their actual canonical workspace manifests/locks.

The browser artifact used Rust 1.99.1 with the explicit installed rustc path,
`cargo build --locked --offline --target wasm32-unknown-unknown
--no-default-features --features product,presentation-webgpu`, followed by
`wasm-bindgen --target web`. `scenario_qualification_server.py` served the
separately packaged artifact and collected the application's existing scenario
receipts. No shared `web/pkg` directory or existing IndexedDB was replaced.
The final seed/reopen pair used fresh loopback origin port 8929.

## Fixes discovered during the second-host exercise

- Bind leaf cache and attention to both selected progression block and source
  revision, so another block cannot reuse a stale painting or selection.
- Show musical previews before the source editor on narrow screens and avoid
  a redundant clipping container around notation controls.
- Size TAB fret-number outlines using actual font bounds, so digits fit between
  the existing 14-pixel string lines. The renderer regression checks those bounds.
- Align the standalone web lock's `spareval` SHA-2 edge with its existing 0.11
  hash family, matching the compatible root-workspace resolution.
- Wait for settled render frames before capture; the earlier short capture wait
  could omit newly placed text even though a later live frame contained it.

Superseded runs remain under the output directory named in `receipt.json`.
They do not qualify the final candidate.

## Limits and next publication boundary

Mere currently links the canonical sibling Woodshed source paths. Publishing the
Woodshed provider and pinning that Git revision in Mere is a separate decision
needed for a standalone Mere clone. The native lock retains its existing Mere
supplier rather than being silently repinned to the consumer checkout.

This is the existing bounded fretted/single-voice notation profile. It does not
qualify general external MusicXML, arbitrary polyphony, advanced engraving,
Graphshell note authoring, playback, other browsers/presenters, or native/device
acceptance. The accepted owner/API decision lives in
`woodshed/design_docs/2026-07-11_stage_set_tools_plan.md`, section
“ADR: shared notation readings in Graphshell (2026-10-19)”.
