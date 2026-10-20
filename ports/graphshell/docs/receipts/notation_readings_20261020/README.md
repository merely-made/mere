# Graphshell notation on the current host — 2026-10-20

Local integration over Mere `427fa4ff8` and Woodshed `111da595`. This reconciles
the existing consumer from `30ab2c35d` and the published `woodshed-notation`
provider. It does not claim publication, deployment or a clean standalone clone.

Chart, Tab and Score read the same saved MusicXML. Event selection carries
between these readings and is bound to the current block/source revision.
Opening or changing a reading does not author. The explicit **Write notation**
action creates missing MusicXML through one ordinary document undo step;
normal Save and fresh-page reopen preserve it.

The read path now accepts valid saved MusicXML independently of older
Card/arrangement metadata. Writing notation and grouping bars retain their
own validation and preserve unsupported source. The new regression exercises
valid music alongside unrecognized historical metadata.

## What is shared

- `woodshedding`: musical data, MusicXML and conversion from resolved material.
- `woodshed-notation`: Sprigging engraving, Bravura asset/license, layout,
  label positions, event rectangles and the existing paint-only playback cursor.
- Host adapters: controls, accessible labels/buttons, selection lifetime,
  document history, saving and playback policy.

The renderer's fallback `accessibility` adapter is now optional, default-on for
Woodshed. Graphshell supplies its own label and event controls, allowing the
current AccessKit 0.25 host to share paint without the provider's direct 0.24
adapter. Renderer output is unchanged in either configuration. Notation belongs
to the web product feature; the read-only viewer excludes music dependencies.

## Evidence

| Check | Outcome |
|---|---|
| Graphshell document tests | 25 passed |
| Djot block parser | 5 passed |
| Renderer, native defaults | 10 passed |
| Renderer, host-provided controls | Same 10 passed |
| Current web product build | Passed |
| Port and web dependency boundaries | Passed |
| Read-only viewer dependency check | No musical model, renderer or Woodshed app/audio packages |

Three final browser workflows passed with no errors or gate failures: initial
write/read/select/undo/redo/save, wide fresh-page reopening, and narrow reopening.
Seven reviewed captures cover 1280×720 CSS pixels at DPR 2 and 455×1220 at DPR 1.
Earlier test pages were closed for narrow qualification; this is not a
multi-page concurrency gate.

`receipt.json` records counts and limits. `source.json` seals 36 inputs;
`package.json` seals the generated browser package, kept outside Git at
`Code/output/graphshell-notation-20261020/pkg-final`. `browser/` and `checks/`
contain the actual observations and logs. The compiler was explicit Rust
1.99.1, with development/test debug information disabled; wasm-bindgen is 0.2.129.

## Reproduce and remaining boundary

The focused commands, with the explicit Rust 1.99.1 compiler environment above:

```sh
# Mere root
cargo test --locked --offline -p graphshell --no-default-features --features editable-blocks block_editing --lib
cargo test --locked --offline -p illume blocks::tests
python3 scripts/check_port_boundaries.py
# Woodshed root
cargo test --locked --offline -p woodshed-notation --lib
cargo test --locked --offline -p woodshed-notation --lib --no-default-features
# Mere ports/graphshell/web
cargo build --locked --offline --target wasm32-unknown-unknown --no-default-features --features product,presentation-webgpu --config ../docs/receipts/notation_readings_20261020/provider-local.toml
```

The override paths identify the tested local provider and need adjustment for
another development checkout. Standalone builds depend on publishing the
renderer refinement and repinning Mere to that revision.
Creation still accepts whole-number 20–400 BPM while the maintainer's range
question is pending. This gate covers the existing fretted/single-voice profile,
not Graphshell note editing, playback, general MusicXML or physical devices.
