# Published notation-provider integration — 2026-10-20

The musical model and renderer resolve from the published Woodshed
implementation `a0ae58f97ca962b29fcd6ae08c9e6aec35d74c61`, with both Graphshell
manifests and lockfiles naming that revision. No provider-local override was
used for these checks. The earlier packets preserve development qualification;
their pending-publication statements describe those earlier runs.

Evidence:

- 26 Graphshell document tests passed against the Git provider with
  `cargo test --locked --offline -p graphshell --no-default-features
  --features editable-blocks block_editing --lib` from the Mere root.
- The standalone web product built with `cargo build --locked --offline
  --target wasm32-unknown-unknown --no-default-features --features
  product,presentation-webgpu` from `ports/graphshell/web`.
- Cargo metadata identifies both musical crates as the exact published Git
  source. The read-only viewer's normal dependency tree excludes them and the
  Woodshed app/audio packages. Port-boundary checks passed.
- A clean `git archive` export of the staged source, omitting ignored and
  machine-local files, passed locked offline web metadata resolution. All local
  packages resolve within the exported tree; both musical crates resolve from
  Git. This is a clean-source dependency check, not a clean compilation.
- The resulting WebGPU package passed `notation_tempo.scn` and a fresh-page
  `notation_readings_reopen.scn`: invalid-tempo refusal, explicit 1 BPM creation,
  undo/redo/save and reopened Score/Tab views. Three captures were reviewed.

The compiler was explicit Rust 1.99.1, development/test debug information and
incrementals were off, and wasm-bindgen was 0.2.129. `checks/`, `browser/`,
`provider-resolution.json` and `clean-source-resolution.json` preserve the
observations. `source.json` and `package.json` seal the source and browser bytes;
the generated package remains outside Git under
`Code/output/graphshell-notation-published-20261020/pkg`.

This publishes feature branches without merging them into main. Existing
fretted/single-voice MusicXML limits remain; this adds no native-window,
physical-device or general MusicXML qualification. Provider tests (210 musical
core tests and the renderer in both accessibility configurations) are recorded
in the preceding packets against the same implementation bytes.
