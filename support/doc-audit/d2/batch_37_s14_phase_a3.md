# Batch 37 — S14 pass, phase A, last seven (records for documents added since 2026-09-05)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/arena/R2_A_RECEIPT.md | current | n/a | 21 | 21 | 0 | 0 |
| mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/RECEIPT.md | current | n/a | 18 | 17 | 0 | 1 |
| mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/execution/R1_EXECUTION_RECEIPT.md | current | n/a | 16 | 13 | 1 | 2 |
| mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/identity/R2_B_C_RECEIPT.md | current | n/a | 23 | 21 | 0 | 2 |
| mere_docs/testing/receipts/2026-09-09_s10_custody_transact/RECEIPT.md | current | n/a | 28 | 22 | 3 | 3 |
| mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/RECEIPT.md | current | n/a | 32 | 31 | 0 | 1 |
| nematic_docs/implementation_strategy/2026-09-15_micron_navigation_plan.md | current | no | 58 | 51 | 5 | 2 |
| **Totals** |  |  | **196** | **176** | **9** | **11** |

**Totals: 7 docs, 196 claims checked (176 holds, 9 stale, 11 unverifiable), 7 contradictions.**

Audit base: Mere `26060e88` (2026-10-05 21:58). Main moved to `24e5ab54`
while the draft was made; the five files that changed touch none of these
documents or their citations. Genet was read at `ee0b314b` and its main,
Turnstone at main `c3b14cb`, Knot at its main, Woodshed at its HEAD.
`archive_docs/` is excluded. No cargo command was run. Two cited models run
entirely in memory and were re-run read-only: `r1_drive_wake_model.ps1`
(prints PASS) and `identity_custody_probe.py` (14 tests, exit 0).

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase A. A read-only subagent (opus) drafted the seven records. Every stale
claim and contradiction below was re-checked in this session: the R1 model's
negative control at the base, the muniment `#[test]` counts at `725b0f35` and
`9ba9f790`, genet `5af76a0cb8c`'s component tree before and after, the
custody-backend receipt's text and `Backend::transact`, the micron plan's
cited lines and `DOC_README.md:547-548`, Turnstone's `effects.rs`,
`nomadnet.rs` and pins, the 30 `navigation/*.mu` pages, the micron commits on
Mere and Knot main, and the thesis brief's R1 row. The "holds" counts are the
draft's. Nothing in the seven documents changes here.

The three files named `RECEIPT.md` keep the active-basename error in
`scripts/mere_doc_judgment_audit.py` until renamed; renaming them is a fork
for Mark in this pass (the records' headers follow any rename).

## mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/arena/R2_A_RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 21 — holds: 21, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The draft checked: genet `ee0b314b` on genet main, with `rust-toolchain.toml`
at 1.97.1 and `ScriptedDom`, `Pins` and the two replacement
`release_subtree(child)` calls at their cited lines; all 19 arena digests in
`artifact-sha256.json`; the source manifest's archive digests and 835
per-file digests; the harness's path patches, seven named tests and 1,000-run
churn loop; the baseline run (exit 101, 5 passed, 2 failed, 551.69 s), the
fence-disabled run (4 passed, 3 failed, 75.20 s) and the retention candidate
(exit 0, 7 passed, 43.73 s). Genet later fixed the defect in `ec5421b7591`.

## mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 18 — holds: 17, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- The S10 receipt (`2026-09-09_s10_custody_transact/RECEIPT.md`, lines 6-9)
  says it supersedes this probe for the production-boundary question; this
  receipt has no pointer forward.
- The transactional conditional API this receipt says product work needs
  (line 25) has since landed as `Backend::transact` (`9ba9f790`, 2026-09-09;
  `crates/eidetic/muniment/src/backend.rs:144`).

### Recommended action

- A dated pointer to the S10 receipt and `Backend::transact`, and the
  `RECEIPT.md` digest in `custody-backend/artifact-sha256.json` updated with
  it, since editing the file changes its hash.

### Notes

The draft checked: all 9 artifact digests; `run.json` (rustc 1.97.1,
`--offline --locked`, exit 0) and `run.log`'s three named tests; the 18
muniment files of the source manifest against Mere `ef2a47fd` in CRLF
form; muniment at that revision (json default, `redb` feature, blake3,
`Backend::apply`); the test bodies against the receipt's bullets; Turnstone's
writer at `86d6a58`. Unverifiable: the initial online dependency fetch.

## mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/execution/R1_EXECUTION_RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 16 — holds: 13, stale: 1, unverifiable: 2

### Stale claims

- Lines 39-42: the negative control "demonstrates that callback-only logic
  sleeps with runnable work". It cannot fail: `r1_drive_wake_model.ps1` sets
  `$callbackRegistered = $false` just before the completion, so
  `$missedCallback` is true by construction and the assertion's
  callback-only half is never exercised. An overstatement from the start,
  not drift.

### Contradictions

- The thesis brief's R1 row (`2026-08-12_family_composition_thesis_brief.md:570`)
  repeats the claim ("Broken one-bit/callback-only controls lose …").

### Recommended action

- Reword item 3 to say the callback-only case is asserted by construction,
  or make the control a real callback path that can fail; the thesis brief's
  row follows. Changing the model changes its digest in the artifact list.

### Notes

The draft checked at genet `ee0b314b`: `DocumentSession::pump` and
`settled` (the cited range 852-858 starts one line late), ortet's shell
pump, frame and redraw, the script runtime's timer, worker and `Drop`
paths, and `LiveryScriptedDocument::pump`. The model re-runs to PASS and its
digest matches the artifact list. Unverifiable: the concurrent-edit
observation and the cache-lock stall.

## mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/identity/R2_B_C_RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 23 — holds: 21, stale: 0, unverifiable: 2

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The draft checked: Mere `cc0b6aa5` and Turnstone `fa4cca57` on their mains;
the model file's SHA-256 (`C87FB4F9…`); muniment's blake3 at the base; the
re-run (14 tests, 0 failures, exit 0); each of the 10 established cases and 4
broken controls mapped to a named test, the tombstone-to-live direction
through the same `move_ref` path. Unverifiable: two concurrent working-tree
remarks.

## mere_docs/testing/receipts/2026-09-09_s10_custody_transact/RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 28 — holds: 22, stale: 3, unverifiable: 3

### Stale claims

- Lines 99-104: "the crate's pre-existing 56 tests … plus 3 new
  `custody_transact_tests` groups" (3 + 3 + 2 tests). Muniment has 51
  `#[test]`s at `725b0f35` and 59 at `9ba9f790`, and
  `custody_transact_tests.rs` adds 8. The total of 59 holds; the breakdown is
  51 + 8.
- Lines 8-9: "that probe's `src/lib.rs` scenarios are ported here as real
  tests". The owner-specific transfer, reopen and self-transfer scenario was
  not ported; `custody_transact_tests.rs` mentions transfers only in doc
  comments, at `9ba9f790` and at the base.
- Line 111: genet's paint component "looks renamed to `webgl-essl`". Genet
  `5af76a0cb8c` (2026-09-07) removed `components/paint` and carved
  `components/genet-compositor`; `components/webgl-essl` existed before it.

### Contradictions

- Its supersession of the 2026-09-08 custody-backend probe is not reflected
  in that probe (the custody-backend record).

### Recommended action

- The breakdown to "51 pre-existing + 8 new (3 memory, 3 redb, 2 refusal)";
  the porting sentence narrowed to the stale-collection and negative-control
  shapes; the paint residual to name `genet-compositor` and `5af76a0cb8c`.

### Notes

The draft checked: `725b0f35` the parent of `9ba9f790`, both on main; the
transaction trait items, the `NotTransactional` default and the boxed
backend at `9ba9f790` and the base; the redb, memory, IndexedDB and zip
backends' behaviour at their cited lines; redb 2.6.3 in the base lock and
its Windows `LockFile` path; the cross-process example, the toolchain at
1.97.1, `.cargo/config.toml` ignored, `ports/muniment-opfs-probe`; Turnstone
main without a `transact` call; the scratch report leaving the cross-process
question open. Unverifiable: the worktree and branch (both gone), the
runtime transcript and the command table.

## mere_docs/testing/receipts/2026-09-20_resource_resolution_probes/RECEIPT.md

- disposition: current
- status line: "" — accurate: n/a
- claims checked: 32 — holds: 31, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- none. It agrees with the thesis brief's R3 results table.

### Recommended action

- No correction needed. Optionally a dated note on three later changes:
  Woodshed `fdc5980` (22 minutes after the receipt) moved Redshank onto one
  shared agent; `bf5923d` (2026-09-22) moved it onto Mere's fetch handle, so
  Redshank no longer declares ureq; Mere now pins genet `bd3e8861`. Its
  `ports/redshank/Cargo.toml` is a Woodshed path, not Mere's.

### Notes

The draft checked: genet `5ae30cad` and `99769450` on main with an empty
`components/netfetcher` diff between them; the three netfetcher manifests at
`5ae30cad`; `iroh-blobs = "0.103"` in the transport crate; Woodshed's ureq
3.4 declaration and 737-package lock; ureq's use-once agent; Redshank's
`ureq::get` calls before the fix and the 196,609-byte (6.6%) figure; the
ranged, held and cost logs; `lock_delta.json`. Unverifiable: the rustc
version, which the logs do not record.

## nematic_docs/implementation_strategy/2026-09-15_micron_navigation_plan.md

- disposition: current
- status line: "Status (2026-09-16): accepted; C1, C1b, N1, P1 and P1b landed, A1 next. Lane 2 of the [smolweb fidelity plan](2026-07-01_smolweb_fidelity_plan.md) ("Document navigation"). Lane 3 (forms) closed on 2026-09-13 with headed receipts; this lane is the next user-visible conformance gap." — accurate: no
- claims checked: 58 — holds: 51, stale: 5, unverifiable: 2

### Stale claims

- The status line leaves out three things that landed 2026-09-16: P1c
  (`f7c9374f`), Cambium's scroll request (`1f5f13e0`, `5dff2f93`) and A1's
  Knot half (Knot `6157a0c` to `b02e15c`, on Knot main, recorded in the
  plan's own Knot status). "A1 next" is half done; the Turnstone half has
  not started: Turnstone main `c3b14cb` still respawns a session for every
  Micron prefix (`src/shell/effects.rs:1147`, `SpawnContent`) and never calls
  `replace_document`, `take_in_page_navigations` or `reveal_anchor`.
- Line 244: "the 31 navigation probe pages and `guide-structure.mu`". The
  `navigation/` fixture directory holds 30 `.mu` pages; with
  `guide-structure.mu` that is 31 pages, 62 packets.
- A1's Turnstone line numbers (lines 295-301) have drifted at Turnstone main
  (the respawn now at `effects.rs:1147`, among others).
- Line 307: "Also the N1 test literal at `src/nomadnet.rs` ~517" is done:
  Turnstone `67aedb3` (2026-09-17) added `navigation: Default::default()`
  (`nomadnet.rs:525`).
- Lines 638-639: "Turnstone pins Mere `5dff2f93` and Knot `b02e15c` together
  next" is overtaken: Turnstone pins Mere `bd5912fb` and Knot `3dfb70b0`,
  descendants of both.

### Contradictions

- The status line disagrees with the plan's own Knot status and its
  Progress entries.
- `DOC_README.md:547-548` describes the plan as "accepted; C1 landed
  2026-09-16, N1 next".
- Line 244's count disagrees with the plan's own later correction (31 of 42
  pages in all).

### Recommended action

- Status to record C1, C1b, N1, P1, P1b, P1c, the Cambium scroll request and
  A1's Knot half landed, with A1's Turnstone half next (Turnstone's pins
  already carry those revisions; its behaviour switch has not started).
- Line 244 to "30 navigation pages and `guide-structure.mu` (31 pages, 62
  packets)"; A1's Turnstone line numbers refreshed; the done `nomadnet.rs`
  item dropped; `DOC_README.md:547-548` to match.

### Notes

An untouched worktree, `Code/worktrees/turnstone-micron-nav-a1`, holds branch
`micron-nav-a1` at `f5ca53a` (already on Turnstone main) plus a modified
`Cargo.toml`. The draft checked: the micron commits on Mere main; the
nematic, inker, document-canvas, document-lanes and Cambium symbols at the
base (`AppCtx::scroll_into_view`, `Host::relayout` draining
`pending_scroll`); 42 fixture pages, 12 with a collapsible heading; every N1,
P1 and P1b test count in the logs under
`Code/testing/mere/micron-navigation-20260916/logs`; the seven A1 commits
on Knot main. Unverifiable: the `sem_impact` and "32 literals" counts, and
Knot's test and control counts. The plan has not been touched since
`fa8c9ad7` (2026-09-16).
