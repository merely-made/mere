# Full written-tempo range — 2026-10-20

The maintainer chose whole-number **1–400 BPM** for explicit Write notation.
This follow-up supersedes the pending tempo decision in
`../notation_readings_20261020/`; that packet remains unchanged.

The shared `woodshedding::notation::draft::score_from_set` converter accepts
the full range for both fallback and saved Card tempos. Graphshell's command
and validation messages agree. Fractional, non-finite and out-of-range values
remain invalid; practice-speed behavior is unchanged.

Evidence:

- 210 woodshedding library tests passed, including new conversion and MusicXML
  round-trip coverage at 1, 19, 20 and 400 BPM through both tempo paths, plus
  invalid fractional, non-finite and out-of-range values.
- 26 Graphshell document tests passed, including full-range command coverage
  and refusal without source mutation.
- The current WebGPU product build, port boundaries and scoped formatting passed.
- `notation_tempo.scn` rejected 0, 401 and 1.5 without source changes, wrote at
  1 BPM, and passed ordinary undo/redo/save. A fresh page passed the existing
  reopen scenario, displaying the saved music as Score and Tab. A direct
  accessibility-tree observation confirmed `<per-minute>1</per-minute>` in the
  reopened document's source. Three captures were reviewed.

Checks used explicit Rust 1.99.1 with development/test debug information and
incrementals off, and wasm-bindgen 0.2.129. The converter command was
`cargo test --locked --offline -p woodshedding --lib` in Woodshed. Graphshell's
document tests used `cargo test --offline -p graphshell --no-default-features
--features editable-blocks block_editing --lib --config <provider-local.toml>`
from Mere. The web build used `cargo build --locked --offline --target
wasm32-unknown-unknown --no-default-features --features
product,presentation-webgpu --config <provider-local.toml>` from
`ports/graphshell/web`.

`source.json` seals 37 inputs, `package.json` seals the generated package kept
at `Code/output/graphshell-notation-tempo-20261020/pkg`, and `checks/` plus
`browser/` preserve the actual output. `provider-local.toml` identifies the
local provider override; its absolute paths need adjustment elsewhere.

This remains local and uncommitted. Both the converter refinement and the
earlier optional renderer adapter need publication before an immutable consumer
repin can qualify a standalone checkout. No new native window, audio/MIDI,
practice-speed or general MusicXML qualification is claimed.
