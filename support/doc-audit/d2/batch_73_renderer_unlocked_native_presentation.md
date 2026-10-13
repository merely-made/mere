# Batch 73: current-supplier native presentation after console unlock

Audit date: 2026-10-12. Incremental judgment after the user unlocked the console
and explicitly requested the unchanged native gate. Source/documentation base
is Mere `e0dae9c99423210b82bc24e0518739d2c9fefe34`; renderer source and supplier
pins match the published qualification at `631355a2f`. This batch supersedes
batch 72 for the three revised documents. Its performance-analysis judgment
remains unchanged. Earlier blocked native receipts and the four inherited D2
findings are preserved; the original snapshot is not modified.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| cambium_docs/technical_architecture/2026-10-11_shared_device_recovery_owner.md | current | yes | 9 | 9 | 0 | 0 |
| cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md | current | yes | 12 | 12 | 0 | 0 |
| DOC_README.md | current | yes | 3 | 3 | 0 | 0 |

Totals: three document records, 24 claims checked, 24 holds and zero stale or
unverifiable claims in the named scopes. No threshold, renderer source, supplier
pin or lockfile changed for this retry, and no CPU build was launched.

## cambium_docs/technical_architecture/2026-10-11_shared_device_recovery_owner.md

- disposition: current
- status line: "Implemented; native controlled-loss and current-frame qualification passed." — accurate: yes
- claims checked: 9 — holds: 9, stale: 0, unverifiable: 0

### Stale claims

None. The final paragraph now includes the executed healthy native presentation
gate instead of leaving its result to the separately linked receipt alone.

### Contradictions

None. Healthy surface presentation does not establish presentation after device
destruction, physical driver-reset recovery or browser asynchronous recovery.

### Recommended action

Preserve those separate gates and the shared owner/detachment contract. Record
later consumer acceptance under its own source and runtime handoff.

### Notes

The eight previously checked architecture/controlled-loss claim groups still
hold against the same source. The ninth is the unchanged current-supplier
native presentation scenario: 53 presented frames and six nonblank, distinct
captures after user unlock. Checked process/command receipts, binary/scenario
identity, scenario assertions, all six PNGs, and explicit qualification limits.

## cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md

- disposition: current
- status line: "the unchanged current-supplier native gate passed" — accurate: yes
- claims checked: 12 — holds: 12, stale: 0, unverifiable: 0

### Stale claims

The prior current-native-presentation pending statement is superseded by the
successful unlocked run. The earlier locked-console observations remain true
historical evidence and are now worded as completed earlier attempts.

### Contradictions

None. The same binary and scenario failed presentation while occluded and
passed after unlock. No source rebuild, error suppression or weakened predicate
was used to obtain the successful result.

### Recommended action

Retain the blocked receipts and frozen identities. Keep controlled readback
rehydration separate from swapchain presentation after loss, and keep browser,
OS accessibility, pooling and physical-reset acceptance open as documented.

### Notes

Rechecked the twelve receipt claim groups from batch 72 against their existing
source/metadata/production/recovery receipts, with current healthy native
presentation replacing the pending conclusion. The new command exited zero
in 17.853 seconds (LaunchServices runner 17.68 seconds), without timeout or
disk refusal. It used Genet `2c7ef73b`, Netrender `c74b7cb2`, provider `2eae6243`,
and the existing binary SHA256
`b4f413755dacece3a863b39e1b43fd6e39e3e0cf709cfd4148622de079f6d030`.
Scenario SHA256
`96f64dd4a425a0978ed366cb3c6823d1cc675f85fe46a82dfa130366409c27cf`
also matches the earlier occluded attempt exactly. The run reports 53 presented
frames, 56 redraws, three unpresented redraws, six captures, zero blank captures
and six distinct capture digests. Authored seed/name, mode, graph selection and
resize/focus assertions passed. All six PNGs show visible product content;
seed hex, dark surfaces, draft caret and reading wrap changes agree with the
scenario. Capture dimensions and hashes remain in the immutable receipts.
Canonical granted the brief runtime window, and Root stopped the exact owned
native app/inhibitor before explicitly returning it. The CPU target lease
remained with Moot. Historical Genet `5fc18369` presentation is not relabeled
as current-source proof. Timing and allocation findings retain their limits.

## DOC_README.md

- disposition: current
- status line: "current-supplier native presentation passed after console unlock" — accurate: yes
- claims checked: 3 — holds: 3, stale: 0, unverifiable: 0

### Stale claims

The two former pending/blocked current-native descriptions have been replaced
with the executed result and its healthy-presentation scope.

### Contradictions

None in these entries. OS/browser acceptance and swapchain presentation after
device destruction remain open; the timing entry remains a limited analysis.

### Recommended action

Keep future acceptance results in their owning receipts and preserve runtime
ownership handoffs when promoting consumer evidence.

### Notes

All three indexed paths resolve. Recovery-owner and production-receipt entries
now match the 53-frame/six-capture result and its limits. The performance entry
still discloses its missing Classic baseline and measured quantities. Unrelated
index prose is outside this judgment.
