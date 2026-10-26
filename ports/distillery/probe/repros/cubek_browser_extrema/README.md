# Cubek browser extrema reproducer

This is the reduction-only extraction of Distillery's first failed MiniLM
browser row. The current qualification uses Burn 0.22.1 and Cubek 0.3.1 to run four scalar
extrema cases through BrowserWebGpu in a dedicated worker:

- a finite maximum;
- an all-negative-infinity maximum;
- an all-positive-infinity minimum; and
- a maximum containing NaN.

**2026-10-03:** the receipts named below from before September record the
pre.2 row. On pre.4 the patched row passes all four cases in headed Chrome
([`receipts/2026-10-03_pre4_chrome.json`](receipts/2026-10-03_pre4_chrome.json),
migration plan §13.30). Pristine pre.4 without the `cubek-reduce` patch has not
been run.

The page captures WebGPU validation errors separately from returned tensor
bytes. This distinction matters because a failed dispatch can still leave a
repeatable buffer that is not a valid result.

The released Cubek source constructs the floating-point reduction identities
by bitcasting literal infinity bits. Chromium constant-evaluates that expression
and rejects the non-finite `f32`. This workspace patches `cubek-reduce` to pass
the bits through a mutable kernel local before reinterpretation while retaining
the infinity and NaN cases that motivated Cubek's identity change.

From this directory, with wasm-bindgen CLI 0.2.129 installed:

```powershell
.\run-repro.ps1 -WasmBindgen C:\path\to\wasm-bindgen.exe
```

Open the printed URL in headed Chromium and choose **Run reduction cases**.
For automation, call `window.cubekExtremaRepro.run()` and inspect both
`result` and `gpu_errors`.

The current lock selects Mere's rebased `cubek-reduce` patch. A pristine
comparison must remove that patch while retaining the same released
Burn/CubeCL/Cubek cohort. CubeCL 0.11.1 now supplies the runtime directly;
the former independent persistence manifest patch is retired. Current
execution results and source identities are recorded in the Burn migration
plan, section 13.48. Older receipts retain their original prerelease versions.

[`UPSTREAM_ISSUE.md`](UPSTREAM_ISSUE.md) contains an issue-ready extraction of
the failure, the generated WGSL before and after materialization, and the exact
validation boundary.

## 2026-10-24 released patch cohort

The [normal browser receipt](receipts/2026-10-24_burn_0221_iab.json) passes with no WebGPU
validation or page-gate errors. The [planted-error control](receipts/2026-10-24_burn_0221_rejection_control.json)
correctly refuses even though its numerical cases match. The current
qualification record and source/lock/bundle hashes live in migration plan
section 13.48. These receipts do not relabel older comparisons or qualify
other browser versions and full products.
