# Caller-owned toolkit embedding qualification

The shared Rootstock embedding contract and optional adapters use the caller's
wgpu 30 Device, Queue, encoder, attachment views and submission. Each appearance
has its own focus, capture, invalidation and GPU lifetime. Retained content and
graph sessions keep their existing source owners.

`embedding-tests.log` records six passing focused contract/input tests. The
actual toolkit consumer command is recorded in `toolkit-bench.log`:

```powershell
$env:CARGO_TARGET_DIR='C:/t/cargo-targets/mere'
$env:CARGO_BUILD_BUILD_DIR='C:/t/cargo-targets/mere/build'
cargo +1.99.0 run -p graphshell --no-default-features --features caller-owned-toolkit-bench,image-decode --example caller_owned_toolkits --locked --offline -j1 -- ports/graphshell/docs/receipts/caller_owned_toolkits_20261013/native
```

The example invokes actual egui-wgpu callbacks and Iced primitives on native
headless targets. It opens verified retained Fleece Knot and Redshank clips
through ContentHost/ContentProducer, retains native HTML and decoded PNG pixels,
and renders the actual Mere Canvas graph scene. It checks clipping, scale,
independent accesses and host-generation resource rehydration. The native audio
payload is verified; this gate does not exercise audio output.

The backend is Netrender's admitted sparse GPU scene lane using outline text and
owned CPU images. Unsupported operations fail admission. Classic's internally
submitting render owner, glyph-atlas parity, external GPU/native media frames,
physical window input/IME/accessibility and actual device loss remain open.
Rehydration here uses a new caller generation on the same Device/Queue after
adapter suspension; it is not a physical device-loss receipt.

egui is 0.36.2. The Iced supplier is exact Git revision
`ab336f6d6403d7206d82c7c2caa46143efde8284`; released Iced 0.14 uses an incompatible
wgpu 27 API. Both selected adapters compile against the stack's wgpu 30.

`root-lock-change.json` and `root-lock-diff.patch` identify the controlled lock
refresh against the named pre-task Git baseline. No existing Git supplier was
repinned. The shader supplier's exact WESL 0.4.0 constraint accounts for the WESL
family downgrade in that lock refresh. The two incidental resolver backups were
removed after recording their reconstructible baseline and final source diff.

Failed first attempts remain as `embedding-tests-initial-compile-refusal.log`,
`toolkit-bench-initial-compile-refusal.log` and
`toolkit-bench-initial-runtime-refusal.log`; later gate logs determine readiness.
