# Batch 72: renderer flattening and shared recovery qualification

Audit date: 2026-10-12. Bounded implementing-agent judgment of the three
documents introduced by Mere `631355a2fcbc6d5e88b1b1040d401a0e0b08bfab`
and their index entries. This supplements the existing ledger without changing
the original snapshot, its expected digest, or the four pre-existing findings.
No new build, GPU/native/browser run, or acceptance-threshold change occurred
during this documentation reconciliation.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| cambium_docs/technical_architecture/2026-10-11_shared_device_recovery_owner.md | current | yes | 8 | 8 | 0 | 0 |
| cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md | current | yes | 12 | 12 | 0 | 0 |
| cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/performance-analysis.md | current | n/a | 7 | 7 | 0 | 0 |
| DOC_README.md | current | yes | 3 | 3 | 0 | 0 |

Totals: four bounded document records; 30 claims checked, 30 holds, zero
stale or unverifiable claims within the named amendments. Older repository
findings and unrelated index prose are not requalified by this batch.

## cambium_docs/technical_architecture/2026-10-11_shared_device_recovery_owner.md

- disposition: current
- status line: "Implemented; native controlled-loss and current-frame qualification passed." — accurate: yes
- claims checked: 8 — holds: 8, stale: 0, unverifiable: 0

### Stale claims

None within this ADR. Its status qualifies controlled native loss and raster
completion; it does not claim current-supplier swapchain presentation.

### Contradictions

None within the checked scope. Surface loss is separate from device loss,
and controlled Device::destroy is separate from a physical driver reset.

### Recommended action

Keep current native presentation, asynchronous browser replacement/scopes,
and comprehensive producer-upload/composition loss qualification open.
Preserve the shared ownership and application-source boundaries.

### Notes

Checked eight claims against Genet `2c7ef73b` and published Mere source and
receipts: one owner/factory per group; immutable generation notices; callbacks
outside owner locks; host-detachment barrier; CPU registration/source retention;
exact-once capture/readback refusal; terminal replacement failure and separate
surface handling; controlled-loss/current-frame evidence with explicit open
browser, physical-reset and upload/composition gates. The linked receipt resolves.

## cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md

- disposition: current
- status line: "Current-supplier native presentation remains pending an unlocked session" — accurate: yes
- claims checked: 12 — holds: 12, stale: 0, unverifiable: 0

### Stale claims

None within the published receipt. The 60-frame, six-capture native pass is
explicitly assigned to Genet `5fc18369`, before the current-frame fence.

### Contradictions

The current Genet `2c7ef73b` raster succeeds while native swapchain acquisition
is occluded by a locked console. The failed stall gate is preserved and is
neither a provider-quality refusal nor successful presentation evidence.

### Recommended action

Rerun current-supplier native presentation only after console unlock and an
explicit serialized runtime handoff. Retain the original pixel, stall and
allocation-pooling predicates and the historical evidence labels.

### Notes

Checked twelve claim groups against the committed supplier qualification,
source freezes, command/exit logs, pixel receipts and native platform diagnostics:
(1) release/supplier identities and five coherent Cargo roots;
(2) Web host/a11y and Graphshell Wasm compilation;
(3) twelve complete panels and twenty-four producer comparisons under unchanged
thresholds; (4) canonical scene/glyph authority, 1/256 tolerance and typed
exhaustion; (5) installed loss callback, one shared replacement and two exact
reader-image rehydrations; (6) retained appearance/authored sources, exact-once
refusal and bounded later factory failure; (7) checked SceneProducer and native
scope controls; (8) historical Genet `5fc18369` presentation identity;
(9) current Genet `2c7ef73b` locked-console/occlusion failure and removed temporary
diagnostics; (10) finite serialized execution bounds with no physical reset;
(11) limited timing observations and pooling still MATERIAL;
(12) actual late round-cap shader refusal before current-frame success and
explicit remaining asynchronous-browser/application qualification.
Distinct producer keys 701/702 do not qualify identical Workshop key namespaces
or swapchain presentation after destruction.

## cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/performance-analysis.md

- disposition: current
- status line: "Identity-matched production replay timing analysis" — accurate: n/a
- claims checked: 7 — holds: 7, stale: 0, unverifiable: 0

### Stale claims

None within the observed replay batch. These historical measurements do not
establish subsequent native completion-fence latency.

### Contradictions

None. CPU raster time, GPU host encode/submit and bounded submit/readback are
distinct quantities; the document does not claim isolated GPU shader timing.

### Recommended action

Preserve input identities and acquire a matching Classic baseline before
reporting a Classic regression ratio. Use repeated warm-session or GPU-timestamp
measurements for broader performance conclusions.

### Notes

Checked seven claim groups against performance-analysis.json and its twelve
fixed-input replay receipts: (1) bundle/capture/dimension/resource/threshold
identity matching; (2) CPU paired width ratios 1.41/1.56 and overall median
1.52; (3) Sparse host encode/submit width ratios 2.11/1.97 and overall median
2.01; (4) absolute Classic timings without a matching baseline;
(5) initialization and bounded readback values; (6) single observations in fresh
sessions, with cold/repeat labels describing capture history;
(7) host-call versus device-execution definitions and the explicit limits on
isolating flatness, variance and thermal effects. No causal or warm-session
performance qualification is inferred.

## DOC_README.md

- disposition: current
- status line: "Shared recovery and production qualification index entries" — accurate: yes
- claims checked: 3 — holds: 3, stale: 0, unverifiable: 0

### Stale claims

None in these three entries. Unrelated index entries remain outside this pass.

### Contradictions

None. The index preserves the difference between controlled raster/recovery
qualification, historical native presentation and the blocked current gate.

### Recommended action

Keep these entries synchronized with their owning documents when later runtime
acceptance or performance measurements are published.

### Notes

All three indexed paths resolve. The recovery-owner entry describes implemented
controlled-loss qualification with open OS/browser gates; the receipt entry
separates current suppliers from historical presentation; the timing entry
retains its measured quantities and missing Classic baseline.
