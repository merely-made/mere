# Ownership repair plan

**Date:** 2026-10-20
**Status:** in progress, 2026-10-20. Mark answered the first round the same day (§5); L1 and L6 are briefed as sessions; nothing numbered by Claude.
**Scope:** turn the full-source audit's registers into repair work, one owner boundary at a
time. The audit (its record is
[2026-10-16_full_source_audit_plan.md](2026-10-16_full_source_audit_plan.md), its registers
under [`support/receipts/authority-success-audit/`](../../../support/receipts/authority-success-audit/))
left 441 documented findings unrepaired, 212 unconfirmed candidates and 1,382 authority
entries that name two or more implementations of one responsibility. No plan owned their
repair. The family promotion plan's boundary programme (execution, supervision, domain
services, custody, observability, application semantics) keeps "Mere's graph ownership
migration and wider protocol/admission consolidation in their existing plans"; this is that
plan.

## 1. The answer this serves

Asked on 2026-10-20 how mere's deduplication and ownership repair should be organised, with
the options: a dated repair plan by owner boundary; the misleading-success class first; fold
into family promotion's boundary slices. Mark: **"Dated repair plan by owner boundary
(Recommended)"**. The lanes below follow the closeout's own repair direction, quoted:

> The most useful repair direction is to finish those boundaries one operation at a time:
> make graph/pane/session or device identity explicit at dispatch; let durable/source owners
> decide admission and revision; keep projections and caches tied to those identities; share
> pure validation and result classification where semantics match. Preserve deliberately
> different privacy, authorization and transport policies. Removing a duplicated owner
> decision is more valuable than splitting a large file while leaving the decision copied.

Everything in this plan that is not Mark's words is a proposal. Rulings are numbered by Mark
when he gives them; until then the open questions in §5 stand as questions.

## 2. The boundaries and their weight

Counts are a keyword grouping of the audit registers made on 2026-10-20 (findings not marked
repaired; candidates; authority entries with two or more implementations whose domain names
the boundary). They size the lanes; the lane's first task is to replace the keyword grouping
with a read of each entry.

| Boundary (closeout's name) | Findings | Candidates | Where they concentrate |
|---|---|---|---|
| Graph and session history | 147 | 95 | graphshell host and web, pictograph, graph-kernel, graphshell-client, chartulary |
| Current projections and caches | 56 | 29 | cambium scenes, esp, platen, document-canvas, pelt desktop |
| Saved versus live content | 59 | 29 | mesh, mien, document-lanes, fetch, document-host |
| Custody lifetimes | 15 | 16 | cambium, pandect, castellan, djinn |
| Transport admission | 28 | 22 | fetch, mesh, inker, errand, distillery probe |
| Host presentation and receipts (the misleading-success class) | 57 | 7 | eidetic-core, cambium hosts, djinn, distillery probe |
| Unassigned by keyword | 79 | 14 | cubek-matmul patch, errand, nematic, cambium, esp |

138 authority entries with two or more implementations sit in the graph, history, session and
projection domains alone (A001 navigation mutation, 3 implementations; A003 graph history and
recency, 4; A004 graph persistence, 4; A008 semantic current-state projection, 4; A016 snapshot
canonicalization and replay authority, 5; A017 cross-graph identity remapping, 5).

## 3. Lanes and done-conditions

Each lane is one owner boundary. A lane reads its slice of the registers first, writes the
list of owner decisions that are copied today and the single owner each should have, and
brings that list to Mark as a question round before changing code. It then repairs one
operation at a time, in its own worktree, with a regression per repaired finding and the
audit's own probe receipts rerun where they exist. A lane is done when:

- every finding in its slice is repaired with a test, or re-documented with the reason it
  stays (a policy the closeout says to preserve, or a scope Mark rules out), and the
  register's status for it is updated in a dated successor receipt, never by editing the
  sealed registers;
- the duplicated implementations in its authority entries are reduced to one owner or
  recorded as deliberately separate (privacy, authorization, transport policy), with the
  owner named in the code's documentation;
- the crates it touched pass their gates in the worktree, the pins still hold, and nothing
  reaches main before that;
- its Progress entry here names the receipts.

**L1. Graph and session history.** First, because it is the heaviest and the closeout's first
example. Start in graph-kernel and chartulary, where the register names snapshot, replay and
navigation owners that disagree: Q009, Q010, Q012, Q013, Q014, Q015, Q016, Q017, Q019, Q031,
Q032 in graph-kernel; Q001, Q002, Q003 in chartulary's stemma and spine; then graphshell host
admission and lifecycle (Q056, Q057) and the pictograph and graphshell-web consumers. The
decision to make explicit: which durable record wins for a node's current address, history
cursor and identity on replay, snapshot load and live mutation (A001, A003, A012, A016, A017).

**L2. Current projections and caches.** Cambium scenes first (eleven findings, nineteen
candidates), then esp and platen. The decision: projections and caches keyed to the identities
L1 makes explicit, invalidated by their owner, never rebuilt from an unsuppressed projection.

**L3. Saved versus live content.** Mesh, mien, document-lanes, fetch and document-host. The
decision: the durable or source owner decides admission and revision; live views do not.

**L4. Custody lifetimes.** Castellan, pandect, cambium and djinn, after the audit's custody
repairs already landed. Coordinates with the family promotion plan's custody process design,
which owns the process boundary; this lane owns lifetimes inside it.

**L5. Transport admission.** Fetch, mesh, inker, errand. Preserve deliberately different
transport policies; remove copied admission decisions.

**L6. Host presentation and receipts.** The misleading-success class across crates: validation
that skips or accepts NaN, benchmark proof that checks one output or a different shape,
ignored-test filters that execute zero tests, fresh files standing in for valid receipts,
cleanup attempts reported as closure. Mechanical, many crates lightly; can run beside any
other lane because it does not move owners.

**L0. Unassigned.** The 79 findings the keyword grouping could not place are read by the lane
whose crate they fall in; cubek-matmul and the other vendored GPU patches are a separate
question for Mark (vendored code, upstream ledger first).

## 4. Rules every lane carries

- Only Mark numbers rulings; a lane proposes and records his words.
- Lanes stop at forks: any copied decision with more than one defensible owner comes back as
  a question with the code evidence, not a pick.
- One worktree and one build directory per lane, never the shared checkout's; at most three
  cargo builds on the machine; gates only on what changed.
- Receipts stay compact: a successor receipt per lane under
  `support/receipts/ownership-repair/<lane>/`, hashes and counts, no copied logs, 100 MB for
  the whole plan.
- The sealed audit registers are never edited; their statuses are superseded by the successor
  receipts, which the audit plan's closeout already anticipates.
- Security and custody code: maintained libraries and standards only.

## 5. Mark's answers, 2026-10-20 (his words; unnumbered until he numbers them)

1. Lane order, asked with the options L1 first with L6 beside it, L1 alone sequential, L6
   first: Mark: **"L1 graph history first, L6 receipts beside it (Recommended)"**. Two
   sessions at once on disjoint files, one slot of the three-task cap spare.
2. L1's first slice, asked with the options kernel and chartulary only, add graphshell host,
   chartulary first: Mark: **"Kernel and chartulary only (Recommended)"**. The fourteen
   storage-side findings (Q009 to Q017, Q031, Q032; Q001 to Q003, Q019) and authority entries
   A001, A003, A012, A016, A017; graphshell admission (Q056, Q057) waits for the owner
   underneath it.
3. Verification bar, asked with the options regression plus the audit's probe rerun, matched
   before/after captures for consumer-visible behaviour, regression only: Mark: **"Regression
   test plus the audit's probe rerun (Recommended)"**. Consumer-visible changes add a
   before/after on the consumer's own tests.
4. The vendored GPU patches, asked with the options ledger only, a seventh local lane, decide
   per finding: Mark: **"Ledger only, out of this plan (Recommended)"**. L0 names them and
   stops; their fourteen findings go to the upstream ledger with the audit's evidence.

Still open, to be put when a lane reaches it: who owns L2 to L5's start order after L1's first
slice lands.

## 6. Findings

- 2026-10-20: the boundary counts above are keyword groupings over the sealed registers; they
  are a sizing aid, not an assignment. 98 of the 147 graph-and-session findings sit in
  graph-kernel, chartulary, graphshell and the canvas crates.

## 7. Progress

- 2026-10-20: drafted after Mark's answer; first round answered the same day (§5); L1 (kernel and chartulary) and L6 (receipts class) briefed as sessions, each in its own worktree, with the bar in §5.3.
