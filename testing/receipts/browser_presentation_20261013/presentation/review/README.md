One fresh whole-branch reviewer examined Mere 17f71d6f..463aea83b and supplier f4147fe..b40f406. It found no Critical issues, five Important code issues and incomplete acceptance gates. One inline fix pass reproduced and repaired all five code findings.

- Pending Auto cancellation left the replacement canvas behind: epoch-owned removal now covers cancellation and stale boot errors; immediate reconnect owns one canvas.
- Provider-start failure retained a live context: startup owns its canvas before supplier begin, releases its context on failure, and lets the supplier choose context attributes. Shader-allocation and asynchronous initialization refusal controls pass.
- Silent image upload GL errors were cached: a temporary texture guard and keyed GL-status refusal now delete failed handles, preserve accounting and allow retry. JavaScript exception cleanup also passes.
- Ordinary uncaptured refusals disappeared: Rootstock retains the backend-neutral failure until successful presentation; browser diagnostics show the refusal, retain the healthy context/backend, and clear on retry.
- Failed gesture construction retained its mounted host: a startup guard owns disposal until every fallible viewer setup step succeeds; context and mirror cleanup pass.

Final native regression: 309 passed, 0 failed, 11 existing ignores. Final artifact passes 32 controls each for Chrome AMD WebGL2, Auto with WebGPU disabled, Firefox and software SwiftShader, plus 26 for native Vulkan WebGPU. Consumer-host root-paint image replacement/removal passes five controls. The bounded read-only viewer itself contains zero image sources, so these are consumer-host and supplier image proofs, not viewer image-loading qualification.

Delivery remains refused: gzip6 3,674,899 bytes versus the unchanged 3,407,872 cap; glue/mount/history pass. The earlier suggested 3.5 MiB cap would still be 4,883 bytes too small. Normal-scale comparisons pass RMS1/max16/alpha0. Fractional-scale comparisons still refuse RMS1 at 1.26449 and 1.19021; alpha matches, and the resized comparison passes. Owner decisions are pending; no limits were changed.

Raw failure receipts remain preserved. The first image-consumer green attempt passed all image assertions but checked disposal before its deferred turn; the corrected test waits for cleanup. The first browser error-observation attempt required the wrong literal substring: the actual attributed refusal was a dimension-contract rejection. The corrected check records that refusal and verifies healthy-context retry. Neither harness correction changed production behavior.

Human screen-reader experience and browser asynchronous WebGPU device replacement remain bounded manual/future gates. Shared-main integration is deferred; supplier and consumer work are preserved on draft branches. Raw inline evidence and source snapshots are archived outside the owned worktrees under `/home/markik/Code/testing/genet/webgl-viewer-20261013/inline-evidence` before cleanup.
