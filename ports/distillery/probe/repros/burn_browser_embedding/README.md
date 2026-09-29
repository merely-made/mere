# Burn browser shared-input binary reproducer

**2026-09-29 pre.4 comparison:** the native comparison passes. In the current
in-app browser, all ten graph and eleven embedding cases pass both with Mere's
backport and with pristine upstream `burn-cubecl 0.22.0-pre.4`. The upstream
pass triggers the migration plan's retirement/retention fork; no decision has
been made. [The comparison receipt](receipts/2026-09-29_pre4_comparison.json)
links exact raw JSON and screenshots. The failure description and experiments
below describe the historical pre.2/browser result from 2026-08-22.

This headed harness extracts Distillery's MiniLM BrowserWebGpu failure from the
artifact, tokenizer, storage, and ESP graph. It retains the eleven embedding
controls that already pass and adds a model-free causal ladder:

- the exact ten-value Burn LayerNorm unit input;
- mean and centering controls;
- one uploaded tensor used by both binary operands;
- two independently uploaded tensors with identical values;
- the resulting variance and LayerNorm;
- an `8 x 384` BERT-width LayerNorm checked against host arithmetic.

On the published Burn/CubeCL row, scalar multiplication and independent tensor
operands pass. `tensor.clone() * tensor` fails, and both exact-unit and
BERT-width LayerNorm return their input unchanged. WebGPU validation scopes stay
empty.

Three patch experiments distinguish the cause:

1. tightening CubeCL's handle mutability count does not restore correctness;
2. allocating a separate binary output does not restore correctness;
3. binding the shared allocation once, aliasing the second logical input to
   input zero, and using a separate output passes every graph and embedding
   case.

The third form is carried by Mere's `support/patches/burn-cubecl` backport.
The before/after headed result is recorded in
[the binary-alias receipt](receipts/2026-08-22_binary_alias_iab.json). A native
WGPU test in this crate checks the shared multiply and exact LayerNorm as a
backend control.

From this directory, with wasm-bindgen CLI 0.2.122 installed:

```powershell
.\run-repro.ps1 -WasmBindgen C:\path\to\wasm-bindgen.exe
```

Open the printed URL in headed Chromium and choose **Run graph cases**. For
automation, call `window.burnEmbeddingRepro.run()` and inspect both `result`
and `gpu_errors`.

## 2026-09-29 follow-up: ruling 410 and the historical control

Mark chose "I suppose A, then B if we can": compare historical pre.2 first,
then retire the patch if supported. The reconstructed pre.2 dependency row
reproduces shared multiplication and both LayerNorm failures in the same current
browser where upstream pre.4 passes all 21 cases. See the
[same-browser comparison](receipts/2026-09-29_pre2_pre4_same_browser_comparison.json)
for raw results, screenshots and the historical source qualification.

`tests/launcher_retirement.rs` adds nine explicitly requested native GPU cases
through public, unfused CubeBackend APIs. Run `cargo test --release --test
launcher_retirement -- --ignored --test-threads=1 --nocapture` from this fixture
with the workspace-approved target environment. The tests preserve real-handle,
fresh-output and unchanged-input checks. They are prepared but not yet executed
at this documentation checkpoint; the burn-cubecl selector still uses Mere's
patch. The earlier pending-choice paragraph is superseded by ruling 410.

## 2026-09-29: selector retired after both comparisons

The nine native controls passed against patched and upstream pre.4, with all
printed numbers identical and retained inputs unchanged. The four owning
workspaces now select the registry burn-cubecl crate on the migration branch.
See [the native comparison](receipts/2026-09-29_launcher_retirement.json).
The historical same-view guard remains in vendored source for provenance;
the test's six-field predicate checks fixture construction only. Remaining
migration gates and main promotion are separate from this bounded acceptance.
