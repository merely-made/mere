# Iconography plan receipts

## 2026-11-10 · F1 blazon source

Mere baseline `28e2350a2`. Guarded Rust 1.99.1
`test --locked --offline -p pictograph --no-default-features --lib -j 4`
passed 26 tests in lane `pictograph-grammar`. The four F1 tests cover 2,304
distinct generated text round trips, the plan's paste example, typed
malformed-text failures and structured serde round trips with slot bounds.
The existing `cells5` digest fixtures passed.

Local evidence lives in `Code/output/pictograph-grammar/2026-11-10-f1-test.log`
and `Code/output/build-storage-receipts/2026-11-10/build-c138529fe1d94ce0a537282a71328f78.json`.

## 2026-11-10 · S1 chrome symbols

Guarded Rust 1.99.1
`test --locked --offline -p pictograph --no-default-features --lib --test symbols -j 4`
passed 26 library tests and two catalog tests. The catalog tests compare all
eight IconVG goldens byte for byte, check both arms around the 32 px boundary,
and verify the twelve-entry palette mapping and ink-only fills. The
`cells5` byte fixtures passed again.

The standalone raster sheet decodes each IconVG once per size and palette
and renders its paths in Chrome 152 Canvas 2D. All 33 pairs have nonempty
coverage, identical alpha and a changed RGB value. The samples are eight
chrome symbols at 18, 32 and 56 px and three address-derived faces at 10,
26 and 56 px. The raster check uses a standalone page with no app UI.

The 1024 × 952 PNG has SHA-256
`6f7ebf0736893c0c58f6d434c4e83b7e8ac2f33bdba873ce8d45551fc412a433`.
The PNG, decoded paths, probe source, renderer and test log are in
`Code/output/pictograph-grammar/s1/`. The guarded test receipt is
`Code/output/build-storage-receipts/2026-11-10/build-4789d578b07c4d578558f7fe6722ab70.json`.

The standalone Graphshell web workspace passed
`check --locked --offline --target wasm32-unknown-unknown --lib -j 4`
on the same toolchain and lane. See `s1/graphshell-web-check.log` under
the lane output and the guarded receipt
`Code/output/build-storage-receipts/2026-11-10/build-c6399389bcf0492ea393240c67cf7603.json`.
