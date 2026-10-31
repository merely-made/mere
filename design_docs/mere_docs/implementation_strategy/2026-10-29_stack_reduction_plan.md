# Stack reduction implementation plan

**Status (2026-10-29): P0 committed; P1 supplier code implemented and qualified on an isolated S-PC branch.** The remaining audit candidates are unimplemented or awaiting their owner-specific trials. Product receiving and integration are separate.

## Goal and source basis

Reduce independent standards machinery while preserving the stack's authority, privacy, durable identity and portability contracts. The full audit is [the research dossier](../research/2026-10-29_stack_ownership_audit/REPORT.md). Each item retains its R01–R41 identity. A candidate becomes implemented only after its owning repository has source, tests and a receipt; it becomes adopted only after consumer pins and their relevant checks advance.

The initial audited supplier revisions are recorded in the dossier. Implementation always starts by comparing current upstream source; do not apply a recommendation to code already removed or replaced. Existing design choices explain requirements but do not prevent a new bounded comparison.

## Constraints

- Work in isolated current-main worktrees and preserve unrelated branches, dirty files and retained evidence.
- Use S-PC over the existing configured SSH authority. Do not copy or mint SSH keys. Host access does not establish GitHub push access.
- Serialize builds with `/home/markik/Code/testing/machine-heavy.lock`; use one Cargo job and one test thread, `MemoryMax=3G`, `MemorySwapMax=0`, `CPUQuota=150%`, and bounded commands. Stop rather than silently raise resource limits.
- Store compiler intermediates in one stable `Code/targets/lanes/stack-reduction-identity/build` lane and retained logs/receipts under `Code/output/stack-reduction-20261029`.
- Use the repository toolchain (currently Rust 1.99.1) for the working gate. MSRV 1.97.1, Wasm, native consumers and runtime/hardware claims have separate gates.
- Use Terra for implementation and Luna for bounded source/review tasks; separate the implementer and reviewer. Keep task briefs and progress durable.
- Plain commits, with the configured maintainer identity and no attribution trailers. Implementation branches remain isolated until integration is requested; commits and qualification do not imply publication.

## Phases and done conditions

### P0: durable audit and remote lane

Preserve the original audit dossier and source/consumer inventories in Mere, index them in DOC_README, and commit this plan. Create the remote Dramatis worktree at current main, record its source basis and run the focused unchanged baseline.

**Done:** audit and plan have commits, exact remote host/branches/paths are recorded, the baseline result and limits are retained, and another session can resume from the task ledger without reconstructing the chat.

### P1: identity primitive and codec reduction

Own plan: `dramatis/design_docs/2026-10-29_identity_primitives_plan.md`.

Task 1 replaces Personae's handwritten fixed-width MAC equality with the already-locked constant-time implementation and adds a persisted-ledger tamper regression. Task 2 replaces Insigne's handwritten Base58 and hex algorithms with small codec adapters, retaining exact key/DID encodings, fixed-size refusals and feature boundaries.

**Done:** both tasks are committed, independent code review is resolved, focused and package/feature gates pass on the exact commit, no unrelated dependencies are updated, and default/all-feature lock behavior is recorded. Runtime side-channel assurance and consumer adoption remain separate.

### P2: other small reductions by source owner

Recheck current source and create exact owner-local task plans for Netfetcher cache policy (R04), bookmark tokenization (R03), header parsing (R05), RDF formatting (R06), color arithmetic (R07), duplicate undo (R08), shared atomic-file mechanics, and the remaining commodity codecs (R02). Finish the Armillary Pool live/public consumer census before removing an API (R09).

**Done per item:** a measured amount of duplicate machinery is removed, independent vectors and refusal behavior pass, immutable identities/wire output retain their declared contract, and the receiving consumer is qualified separately. A source inventory or candidate library's documentation is not a passing adapter receipt.

### P3: standards-library compatibility trials

Feeds, cookies, MCP, bounded MessagePack/protobuf, WebVTT, WebIDL, ICU providers, text/diff storage and llguidance are separate trials. Keep their source/provenance, bounds, runtime/realm, no_std/Wasm and private-buffer requirements explicit. Use the audit's deciding tests rather than a library feature checklist.

**Done per trial:** an adapter and comparison corpus prove the required behavior, target/dependency costs are recorded, and a retain/replace decision names the actual removed code. Failed trials are preserved as evidence rather than presented as completed migrations.

### P4: strategic ownership comparisons

Selectors, Streams, ESSL, ordinary GUI controls, dual VM support, model architectures, directory transactions and radio scope require independent bounded comparisons. These are optional architectural decisions, not automatically authorized replacements merely because they appear in the audit.

**Done per comparison:** one concrete competing path demonstrates the same essential product behavior, removed maintenance surface and target/runtime costs are measured, and the resulting decision is explicit. Retaining an implementation requires a named benefit and a maintained qualification boundary.

## Findings

- 2026-10-29: the audited MAC loop and Insigne codec copies are still present at Dramatis `6711105e0284adb3e881fc9aebcc56f25000c1ca`.
- 2026-10-29: constant_time_eq 0.4.2 and hex 0.4.3 already occur in the supplier lock. bs58 needs a narrowly scoped new resolution. Making a transitive crate direct must update the package dependency edge without broad lock churn.
- 2026-10-29: S-PC had near-zero load, no compiler jobs and approximately 4 GiB available when selected. Recheck before each substantial run; an initial idle snapshot is not an exclusive reservation.

## Progress

- 2026-10-29: audit dossier transferred to the S-PC Mere lane; isolated current-main worktrees established. Initial gate invocation exposed systemd's default working-directory behavior before any compile; subsequent runs specify the checkout explicitly and retain the failed launch log.
- 2026-10-29: P1 supplier changes committed in Dramatis at `c15a65b` and `27b731f`, with both task reviews approved. The owning plan records source and test boundaries; retained evidence lives in `dramatis/support/receipts/identity-primitives-20261029/`. Whole-branch review is recorded separately. No product dependency pins were advanced.

## Decisions

The controller's initial scope is P0 and P1. The audit is a ranked investigation, so broad toolkit, engine, storage-format and protocol replacement needs its own concrete comparison. The rest of the queue remains explicit rather than silently committing to all 41 items.

## Remote continuation

- Host: `s-pc`, selected after read-only load/process/memory checks. Builds remain capped and serialized.
- Mere lane: `/home/markik/Code/worktrees/mere-stack-reduction-20261029`, branch `stack-reduction-audit-20261029`.
- Dramatis lane: `/home/markik/Code/worktrees/dramatis-stack-reduction-20261029`, branch `stack-reduction-identity-20261029`.
- Supplier plan: `dramatis/design_docs/2026-10-29_identity_primitives_plan.md`; per-task reports, review verdicts and the active ledger are retained on that lane.
- Receipts: `/home/markik/Code/output/stack-reduction-20261029` on S-PC and the committed Dramatis receipt directory.
- Branches are committed and preserved for continuation, without merging shared main or pushing GitHub. Local copies of the same branches can be fetched over the configured SSH host without changing a checkout.
