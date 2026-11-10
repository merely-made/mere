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

## 2026-11-10 · S2 Hocket icon pack

Mere baseline `dda5a73d0`. Guarded Rust 1.99.1
`test --locked --offline -p pictograph --no-default-features --features pack --lib --test symbols --test icon_pack -j 4`
passed 32 tests. The two pack tests cover rejected IDs and destinations,
repeatable output, all PNG dimensions and visible ink, and exact PNG payloads
in both container directories. The `cells5` and S1 byte fixtures passed.
The guarded receipt is
`Code/output/build-storage-receipts/2026-11-10/build-c1f1fe1518d24558baf3e090fe6f90f9.json`.

Two runs of the frozen pack binary on macOS 15.8.1 produced identical sets
of 12 files. The first set is in `Code/output/pictograph-grammar/icons/hocket/`.
The repeat set, binary, per-file SHA-256 list and decoder logs are under
`Code/output/pictograph-grammar/s2/`. The manifest records IconVG SHA-256
`28ab3cecb0fbc6842999033e2f2824578121b77bb40aae2ad4ae9a897b3fa3f1`.

Preview opened the generated `icon.icns`. Apple's `iconutil` extracted the
ten entries and ImageIO independently decoded those ten ICNS images and all
seven ICO images. The configured SSH endpoint on o-pc timed out on two
attempts. Windows Shell acceptance remains open.

The standalone Graphshell web workspace passed
`check --locked --offline --target wasm32-unknown-unknown --lib -j 4`.
The lane log is `s2/graphshell-web-check.log` and the guarded receipt is
`Code/output/build-storage-receipts/2026-11-10/build-90fb829f78bf48309ab79d4d5e5a8f22.json`.

## 2026-11-10 · S2 Windows Shell acceptance

`o-pc` answered as `O-PC` on Windows `10.0.26320.0`.
`SHGetFileInfo` loaded the actual Hocket ICO from disk and returned a
32 × 32 icon containing the amber Hocket mark. The Windows SHA-256 was
`2ad4b1cb75b59a60cb8de7fe2e7b059316d616ece27fd2ca344e9fce8e7979d2`,
matching the retained macOS-tested artifact. The check source, JSON
and decoded PNG live in `Code/output/pictograph-grammar/s2/o-pc/`.
The PNG SHA-256 is
`dcb06a5c1a0de6468c48cbcee7e85c4c0e4cbf1a6030f011e5e1c39f90f26009`.

## 2026-11-10 · W1 named action icons

Graphshell supplier `7b913edc`, adopted by both Mere workspaces. Guarded
Rust 1.99.1 `test --locked --offline -p chirograph -p graphshell-client --lib -j 4`
passed 63 Chirograph and 70 Graphshell client tests. The wire test verifies
the exact legacy JSON string, named round-trip and unchanged payload
composition. The log is `Code/output/pictograph-grammar/w1/chirograph-test.log`
and the guarded receipt is
`Code/output/build-storage-receipts/2026-11-10/build-f73811598f6545e1a8507ce098754d10.json`.

The receiving host lane passed 20 Graphshell-host and 26 Pictograph tests
under `test --locked --offline --no-default-features -p graphshell-host -p pictograph --lib -j 4`.
The action view test covers decorative icon markup, label escaping and
exact text-only output for reserved, unknown and case-mismatched names.
The live endpoint and existing `cells5` tests passed. The lane log is
`Code/output/pictograph-grammar/w1/host-test.log` and the receipt is
`Code/output/build-storage-receipts/2026-11-10/build-2dcb5ff989bb4ef38f3a99d3f57b0461.json`.

The standalone Graphshell web workspace passed
`check --locked --offline --target wasm32-unknown-unknown --lib -j 4`.
The lane log is `Code/output/pictograph-grammar/w1/graphshell-web-final.log`
and the guarded receipt is
`Code/output/build-storage-receipts/2026-11-10/build-2382c37ed38340489dbbf08f8a010d4d.json`.

The default native resident library passed
`check --locked --offline -p mere-resident --lib -j 4` on Mere base
`b8cada27f`. The subsequent upstream facade removal deleted an unused
workspace member and its lock row. The resident check log is
`Code/output/pictograph-grammar/w1/mere-resident-check.log` and the guarded
receipt is
`Code/output/build-storage-receipts/2026-11-10/build-1144841dc85f4486bec7cec8267e319c.json`.
