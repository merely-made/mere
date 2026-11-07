# Stack reuse review and branch comparison

**Status, 2026-10-30: isolated COSE comparison complete; hold production adoption. Other implementation remains paused.** The [audit](../research/2026-10-29_stack_ownership_audit/REPORT.md) keeps R01–R41 for traceability. It is a source review and candidate list, not an instruction to implement every item.

## Decision rule

Preserve pure Rust, transport/protocol modularity, retained data and projections, stable component identity across Graphshell, accessibility/hit testing/input, and the unified wgpu device. R29 (ANGLE), R30 (toolkit replacement) and R33 (C++ DSP suppliers) are struck. Other unproven architectural alternatives are outside the reuse queue. A future architectural argument must account for the whole capability and migration cost before proposing a change.

Choose reuse for an actual obligation removed at lower total cost. Compare remaining adapters, accepted/refused inputs and security semantics, complete consumer dependency/features, default/cold build and disk/artifact cost, MSRV 1.97.1, Wasm and applicable no_std targets. Established status, Rust language, sibling use and deleted lines are insufficient by themselves. Prefer compatible existing primitives and internal consolidation.

Small accepted substitutions need a focused diff, independent existing vectors and relevant results. Do not require a new plan, receipt directory, phase ledger or separate reviewer for each helper. More consequential adapters need evidence proportionate to their risks.

## Current comparison branches

| Repository / branch | State and evidence |
|---|---|
| Mere `stack-reduction-audit-20261029` | Documentation corrections based on current main `c6290142866eefead385e728c5c5da24ac6f624a`. Original audit/plan commits `deae90c09` and `54e09dd11` already reached main before branch-only steering. This correction is branch-only. |
| Dramatis `stack-reduction-identity-20261029` | Published comparison branch. Personae MAC and Insigne Base58/hex changes are `c15a65b` and `27b731f`; pre-merge Linux supplier gate: 121 passed, 4 existing local-sshd ignores. Source reviews followed through `fd2c3a5`. Upstream `051e95a` was merged at `e79db34`, but its queued gate was stopped. Do not call the merged head requalified. No consumer pins were advanced. |
| Dramatis `cose-insigne-comparison-20261030` | Published standalone codec/comparison at `d6bfdcd`, based on `6aa0daa`. Native working/MSRV and Wasm codec checks pass within the documented scope; production files are unchanged. |
| Eidetic `stack-reduction-bookmarks-20261029` | Plan-only branch based on `f3424a9`; current main observed at `20c9793`. No parser, manifest or lock changes. Unchanged baseline was stopped while fetching existing development dependencies, before compilation/tests. Tokenizer adoption remains unchosen. |

R01 and R02 remain open across the family. The audit records four additional verified authentication comparisons in Castellan/Murm; Insigne is only one codec owner. Finish a pattern-based census before claiming either finding complete. Existing supplier receipts remain historical evidence of their exact tested revisions, not qualification of newer heads or downstream products.

## Candidate follow-up

The 2026-10-30 [security reuse research](../research/2026-10-29_stack_ownership_audit/storage-identity-inference.md#security-reuse-follow-up-2026-10-30) is complete as a source/specification comparison. The authorized [COSE comparison](https://github.com/merely-made/dramatis/blob/d6bfdcd20fef38ed1edb5b81660193e39f164dad/support/experiments/cose-insigne-20261030/README.md) is complete outside production paths: 16 tests pass on working/MSRV Rust, plus both Wasm codec checks and seven independent wire fixtures. Retain the experiment and hold production adoption: modest potential encoder deletion leaves substantive ID, checked-proof and retained-history migration, and application authority remains local. Bitwarden is held pending a concrete net-deletion case; no production migration or dependency adoption is selected.

Prefer checking existing in-family reuse (Backstep, already-used RDF/color primitives), the comparison/codec patterns and unused Pool consumers. Cache policy, header syntax and bookmark tokenization have concrete coverage gaps, but their library choices still need benefit and cost comparison. Luggage's staging trust concern is separate from selecting an updater framework. No production implementation is running.

## Remote lane and cleanup

The COSE comparison ran locally in `/home/markik/Code/worktrees/dramatis-cose-comparison-20261030` because S-PC remained unreachable. It used the existing shared lock and unchanged caps. Preserve this branch/worktree for comparison alongside the earlier lanes.

Recording continuation (2026-10-30): S-PC was unreachable, so documentation only was updated locally in `/home/markik/Code/worktrees/mere-stack-reduction-record-20261030`, using the same comparison branch. No builds or product changes were performed. Fetch the published branch before resuming either checkout.

S-PC worktrees remain under `/home/markik/Code/worktrees/{mere,dramatis,eidetic}-stack-reduction-20261029` on the branches above. Preserve them for comparison. Push branches only. After comparison, retain any needed evidence and unique work, check ancestry and then explicitly choose which branches/worktrees to clean; no automatic deletion is scheduled.

If builds resume, serialize through `/home/markik/Code/testing/machine-heavy.lock`: one Cargo job/test thread, MemoryMax=3G, MemorySwapMax=0, CPUQuota=150%, bounded commands. Do not raise caps. Existing target/output lanes and SSH authority remain in place. No new keys, toolchain changes or attribution trailers.
