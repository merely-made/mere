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

### Boundary cues, 2026-10-26

These are the acceptance questions from the ownership assessment Mark brought on 2026-10-26 (an
agent's design review, not his words; unnumbered), mapped onto the sealed registers so a lane
answers each with a test instead of re-deriving it. Each is a property one owner boundary holds
or does not; the register ids are where it fails today, read against the audit's basis commits
and the L1 and L6 successor receipts (Q013 is out, repaired by L1). The controls already in the
tree are `ObservedCommand` and `LiveCheck` in
[`live_graph.rs`](../../../crates/system/pandect/src/live_graph.rs), which name existence, record
digest, shown resource and tags separately; the staged candidate in `GraphSession::transact_change`,
which installs graph and history only after storage acknowledges; and `source-edit`'s stamps and
receipts, which describe an observation without claiming the host applied it. A cue holds when
every consumer of its boundary either enters a path of that kind or is recorded as a different
operation with its own rule. Findings whose source has left mere are marked with their repository.

- **Can two windows overwrite one another without detecting the relevant conflict?** L1, L2.
  Today: Q253, two Forme sessions on one slot both acknowledge and reload loses the first edit,
  while the presentation session refuses the same stale baseline; Q214, Djinn's dial-hint writer
  and its pair/unpair writers each load, change and save one settings file with no revision
  check; Q061, a second client request replaces the first pending operation in `SessionCore`;
  A004 and A011, four graph persistence paths and two writable address representations that are
  not reconciled; A028, public assertion and declaration writers outside the one claimed
  `GraphDelta` path. The check is as narrow as the operation: refusing a title edit because an
  unrelated node moved is a false conflict, and `LiveCheck`'s granularity is the model for Forme,
  settings and client slots.
- **Can a projection with incomplete data save itself as the complete graph?** L1.
  Today: Q012, import provenance rebuilds authoritative records from an unsuppressed projection
  and erases suppressed memberships; Q017, the kernel store reports a successful save that drops
  Surface pins and facets, and A004 records the same omission in Pandect's session graph store;
  Q257, a projection save acknowledgement marks a replacement draft clean; A067, the presentation
  acknowledgement is client progress, not a durable receipt, and the protocol prose says
  otherwise. Pandect's `NeedsCompleteOperation` and the resident's filtered publication are the
  seam: a reader holding less than the complete record is not a writer of it.
- **Can a late result alter a replacement session?** L2, L3.
  Today: Q256, retained presentation rules keep the boot-time graph owner after the active graph
  is replaced; Q257, the old save completion lands on the replacement editor; Q263, a closed
  applet handle still reviews and closes later mounts; A683, transport generations and logical
  host identity are not one owner, so Q270 and Q271 leave stale callbacks and retirement
  unfenced; A518, two cancellation paths with one settlement. Cancellation is not the defence:
  the receiving owner checks that the result still names the target and observation it was asked
  for.
- **Can a failed write advance in-memory history?** L1, L3.
  Today: Q240, a rejected capture restores browsing memory but leaves the graph edits pending in
  `MereHost`, with A550 and A551 naming the missing candidate/rollback seam and the batched apply
  that cannot strengthen a non-atomic backend; Q047, Djinn's door returns Accepted from an
  in-memory queue it later drains and drops; Q234, a failed browsing-memory flush discards the
  unsaved segment (`repos/eidetic`); Q380 and A1119, a failed group-session save is settled by
  the same-session retry (`repos/eidetic`). A565 is the control: resident observed edits already
  go through `transact_*`, and their refusal tests inspect graph, journal, revision and bytes.
- **Can retrying an acknowledged operation apply it twice, or skip it?** L3, L4, L5.
  Today: Q021, an interrupted identity adoption moves the device identity and then skips the
  remaining personas on retry; Q050, a failed pre-key publication is not retried on the next open
  despite the recovery claim; Q380, the retry settles what was never saved; Q362, repeated
  vouches mint Standing each time (`repos/gemot`); Q375, overlapping watches wake twice for one
  subject (`repos/eidetic`). A023 is the repaired control for epoch-revocation idempotence. A
  cross-owner operation such as a document save plus a graph publication needs a stable operation
  identity and an explicit intermediate state before a retry is answerable at all.
- **Can a permission revoked during deferred work still be used when the effect occurs?** L4, L5.
  Today: Q060 and A056, the shared host loop pushes notices before its only authority check and
  idle timeouts repeat the path, so an expired or revoked grant keeps receiving; Q374, a
  registered watch keeps revealing changes after its read grant expires (`repos/eidetic`); Q354,
  a capsule page publishes a deferred navigation after revocation or Stop (`repos/gemot`); Q418,
  a successful revoke cannot interrupt an in-flight event wait in Signalman; Q201 and Q219, a
  leased result or an expired lease still publishes. Holding a Rust value or a deserialized
  identifier does not establish permission; the check belongs at the effect's own admission seam.
- **Could a second host edit the same session by implementing presentation and request
  translation alone?** Every lane; the same question as the
  [stack lift plan](2026-10-14_graphshell_stack_lift_plan.md)'s second-host receipts.
  Today: Q316, native and browser hosts give one leaf different accessible names because
  precedence lives in each lowering; Q255, a second string-to-command table silently redefines a
  command id; A015, seven copies of write-with-backup mechanics with different rollback
  behaviour; A062, a retained resume loop beside `SessionCore`; A846, the browser smoke
  hand-assembles a host path instead of using the maintained one; A993, two authority adapters
  with different conflict precedence by design, which is the shape a deliberate difference takes
  and is recorded as one. A host that has to invent save ordering, history repair, permission
  interpretation or stale-result handling is reporting that the boundary beneath it is not
  finished.

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

L1 card round, 2026-10-20 (Mark's words, unnumbered): chartulary is now in repos/eidetic, pinned by rev: **"Branch in eidetic + repin (Recommended)"**. Current address on load: **"Stored node address (Recommended)"**. Adding an existing node id: **"Insert-if-missing live too (Recommended)"**. Creator after owner deletion: **"1, or 2 if no ancestor"** (re-parent to the deleted owner's creator, else clear).

Still open, to be put when a lane reaches it: who owns L2 to L5's start order after L1's first
slice lands.

Open forks from L1's second pass, 2026-10-26 (questions with the code evidence; each lane stopped
rather than picked). Mark ruled all five the same evening, one at a time with the evidence; his
words are under each, unnumbered until he numbers them:

- **Q010, malformed facet JSON on replay.** `CapturedDelta::to_delta` in graph-kernel
  `graph/capture.rs:382` turns a facet record whose `value_json` does not parse into `None`,
  the same path the loader takes for deliberately obsolete thumbnail and favicon records
  (`:351`), so a corrupt journal record is dropped and the replayed graph differs from its
  journal with no report. Either the journal is refused at load, as Pandect already reports
  Corrupt for other reopen failures, with obsolete imagery kept as an explicit skip; or it loads
  and reports how many records were dropped. The first makes one corrupt record unloadable; the
  second keeps the divergence but makes it visible.
  Mark, 2026-10-26: **"Refuse at checked load (Recommended)"**. A malformed record is corruption:
  checked replay stops there and open reports it, as a refused write does; obsolete imagery
  becomes an explicit skip.
- **Q017, the kernel `store` feature.** `graph-kernel/src/store.rs` saves and loads `graph.json`
  from the snapshot alone, dropping Surface pins and facets while reporting success. No manifest
  in the workspace enables `store` (A004 found only integration-test consumers), and Pandect's
  `GraphSession` already persists facets with checkpoints. Either the feature and module go,
  leaving `GraphSession` as the one persistence owner of A004's four; or they stay and gain a
  facet sidecar, which the triage warns is another partial-save authority. `mere-kernel` 0.0.2 is
  on crates.io, so removal is a public-API change in the next release.
  Mark, 2026-10-26: **"Remove it (Recommended)"**. The feature and module go; `GraphSession` is
  the one persistence owner; a public-API change for the next `mere-kernel` release.
- **Q031, identical assertion copies on parallel rows.** `frozen_selection.rs:162` accepts a
  second identical handle on another row, `frozen_link.rs:220` refuses it as a duplicate, and
  `try_from_snapshot` admits the graph. One validator for assertion handles across snapshot
  load, selection freeze and link freeze (A012, A016) is the repair either way; the question is
  whether identical copies on parallel rows are legal graph state, in which case link freeze
  accepts or dedupes, or not, in which case snapshot load and selection freeze refuse.
  Mark, 2026-10-26: **"Not legal (Recommended)"**, with his clarification that the freezes exist
  "to get a snapshot for representing a node's content in the graph". So one handle, one row
  is a rule of the live graph, enforced at the write path and checked load; a freeze copies legal
  rows faithfully, and the one shared validator checks that the frozen record is a canonical copy
  rather than re-deciding legality. Existing copies need a migration or a documented dedupe on
  load.
- **Q016's edge replays.** The literal repair makes `apply_graph_delta` refuse a replayed
  literal handle that checked open would refuse, but `ReplaySetResourceEdgesByIds` and
  `ReplaySetEdgesByIds` still accept a statement handle equal to a held Resource literal handle:
  `set_resource_edges_between` (`resource.rs:318`) compares edges only against edges, and
  `set_edges_between` (`edge_ops.rs:921`) checks nothing, while `legacy_resource_migration.rs:815`
  removes Surface authority before the exact-handle admission check and so passes through a
  state where both strata hold the handle. Either the edge arms run the same validator, and
  migration is given an explicit staged state that the check tolerates; or edge replays are
  declared out of this invariant's scope. Until one lands, "any applied delta can be reopened"
  holds for literal writes only.
  Mark, 2026-10-26: **"Yes, join the invariant (Recommended)"**. `ReplaySetEdgesByIds` and
  `ReplaySetResourceEdgesByIds` run the shared validator; legacy migration gets an explicit
  staged state the check tolerates until it completes.
- **Q031's legacy baselines with copies** (L1, 2026-10-26, the lane stopped rather than picked).
  With checked load refusing a repeated handle, the caller-qualified legacy migration dedupes the
  identical parallel copies of the Resource rows it builds for each claim and records each dedupe
  as a `LegacyMigrationNote` on the owning Resource (`legacy_resource_migration/copies.rs`). Two
  baselines still fail the migration's final checked load with the shared reason, confirmed by
  throwaway probes: identical copies of a Surface-placed claim (for example `UserGrouped` on
  parallel Surface rows), whose dedupe would belong in the Surface write-back after "Remove
  surface authority first" (`legacy_resource_migration.rs:823` onward); and a baseline whose
  Resource rows already hold copies that the migration never placed, where a dedupe rewrites rows
  with `baseline_effects` consequences. Either the dedupe extends to those two, at whole-row or
  statement granularity; or those baselines are refused and migrated by hand. Also open: the
  frozen link still refuses an empty handle that checked load accepts.
- **Q010 in legacy migration** (L1, 2026-10-26, the lane stopped rather than picked). Checked
  replay now refuses a malformed facet record, but the caller-qualified legacy migration replays
  its journal under `OnRefusal::Drop`, and `legacy_resource_migration.rs:395` still drops such a
  record with `if let Some(delta) = entry.delta.replay_delta_as(..)`. Either migration refuses it
  as a `LegacyMigrationError`, as `check_typed_record` at `:398` refuses an invalid typed resource
  capture; or it keeps dropping, since the migration is caller-qualified and its diagnostics could
  count the drop. Once decided, the three drop-both adapters in `capture/replay_outcome.rs` go.
- **The workshop's home**, raised by the lanes-to-tabard cut (stack modularity map cue,
  2026-10-26). `tabard-workshop` is the only crate under `crates/cambium/` that links the
  document stack; it was placed there on 2026-10-08 so applications other than the Tabard port
  could embed it. Either it stays and a toolkit grouping carries the document stack; or it moves
  beside its desktop host under `ports/tabard/`, still embeddable by path; or its reader and
  stylesheet specimens go behind features. The boundary check prints the carried link until one
  of these lands.
  Mark, 2026-10-26: **"Beside its host (Recommended)"**. `tabard-workshop` moves to
  `ports/tabard/workshop` with its workspace path updated; still embeddable by path; the toolkit
  directory stops carrying the document stack and the guard's carried list empties. Later the same
  evening: **"the guard refuses dev-only crate-to-port links too"**; the web host's
  `shared_recovery` example moves beside the workshop.

## 6. Findings

- 2026-10-20: the boundary counts above are keyword groupings over the sealed registers; they
  are a sizing aid, not an assignment. 98 of the 147 graph-and-session findings sit in
  graph-kernel, chartulary, graphshell and the canvas crates.
- 2026-10-20 (L6): a read of all 465 findings replaced the keyword count of 57 for the
  misleading-success class: **95 in class**, 30 production code in mere, 39 tooling in mere,
  11 in the vendored GPU patches, 15 whose source has since left mere (eidetic-core, luggage and
  stickleback now in `repos/eidetic`; esp in `repos/esp`; mien in `repos/gemot`; errand), plus 12
  related findings whose fix needs an owner choice and belong to L1 to L5. The register holds 39
  findings under `support/patches` (Q424 to Q462), not the fourteen named in §5.4; the eleven in
  the class include burn-remote, cubecl-runtime and cubecl-wgpu, which the ruling's list did not
  name. Q054 (djinn-testkit) was already repaired on main by `2e683fcfc`. List and receipt:
  [`l6/2026-10-20_l6_receipt.json`](../../../support/receipts/ownership-repair/l6/2026-10-20_l6_receipt.json).

## 7. Progress

- 2026-10-20: drafted after Mark's answer; first round answered the same day (§5); L1 (kernel and chartulary) and L6 (receipts class) briefed as sessions, each in its own worktree, with the bar in §5.3.
- 2026-10-20: L1 repaired Q013 (stored address wins on load, regression plus kernel suite 474/474, receipt `support/receipts/ownership-repair/l1/2026-10-20_l1_receipt.json`). Remaining in the slice: Q009, Q010, Q012, Q015, Q016, Q017, Q031, Q032 in the kernel, Q014 re-document only; Q001, Q002, Q003, Q019 wait on an eidetic branch and repin.

- 2026-10-20 (L6): Mark's card round, his words: first crate **"djinn-testkit first
  (Recommended)"**; moved code **"Record and hand off (Recommended)"**; historical dated
  scripts **"Re-document, keep bytes (Recommended)"**; GPU patches **"Ledger all 11
  (Recommended)"**; Q052 **"Narrow claim, report all (Recommended)"**. First crate landed on
  branch `l6/misleading-success` (not merged): Q052 repaired (verdict extracted to
  `ports/djinn/tests/residue_verdict`, unjudged host blocks printed with their reason, regression
  `residue_verdict_classifier` 5 tests, mutation control fails 4 of 5, the audit's probe rerun 5
  passed), Q054 verified in the worktree (djinn-testkit 14 passed), GPU findings added to the
  upstream ledger as item 14. Receipt: `support/receipts/ownership-repair/l6/2026-10-20_l6_receipt.json`.
  Remaining by consumer risk: support/testing runners (Q422, Q463 to Q465), script-host
  build.rs (Q371), cambium-genet-winit-host receipts, pelt and graphshell receipt paths, the 30
  production findings, probes. Not run: the full `knot_residue` run (heavy); its verdict is
  unchanged, only its reporting.

- 2026-10-26: boundary cues added to §3 from the ownership assessment Mark brought the same day,
  each mapped to register ids and a lane; Q013 excluded as repaired (L1 receipt). No code change.

- 2026-10-26 (L1, second pass): Q009, Q012, Q015, Q016 and Q032 repaired in graph-kernel, one
  regression each, on main. Q009 follows Mark's insert-if-missing ruling. Q016 needed a second
  step: a write the kernel refuses is now `GraphDeltaResult::Refused`, the journal's checked
  replay paths stop there and report `ReplayRefused`, and Pandect's open reports Corrupt and
  `graph_at` None for a journal carrying a colliding record, which its own tests already
  required; tolerant scratch replays drop the entry and continue. Kernel lib suite 484/484
  (477 before), Pandect lib suite 340/340. Forks recorded in §5: Q010, Q017, Q031, Q016's edge
  replays. Q014 stays re-document only; Q001 to Q003 and Q019 wait on an eidetic branch.
  Receipt: `support/receipts/ownership-repair/l1/2026-10-26_l1_receipt.json`.
- 2026-10-26 (L1, Q017 closed by removal): the kernel's `store` feature and module are gone per
  Mark's ruling the same evening; `GraphSession` is the one persistence owner (A004 drops to
  three paths). The two reopen integration tests that were gated on the feature now run against
  a test-only `graph.json` helper. Kernel lib 484/484, integration 2/2. Receipt updated.
- 2026-10-26 (the workshop's home, landed): `tabard-workshop` moved from `crates/cambium/` to
  `ports/tabard/workshop` per Mark's ruling; workspace path, the toolkit README map, the port
  README, the architecture doc link and one test's relative asset path updated; the 2026-10-08
  theme-modes decision keeps its text with a dated note. `scripts/check_port_boundaries.py` now
  needs no toolkit carve-out. A proposal to print rather than refuse a dev-only link from a crate
  to a port (the web host's `shared_recovery` wasm example mounted the workshop) was put to Mark
  and reversed the same evening: **"the guard refuses dev-only crate-to-port links too"**, so the
  example moved into `ports/tabard/workshop/examples/` with its wasm-only dev-dependencies (the
  workshop's native host and window stack moved behind `cfg(not(target_arch = "wasm32"))`), and
  the guard passes with nothing carried. Type-check of the workshop with its tests and the Tabard
  desktop passed. The move also exposed bit-rot independent of it: the example did not compile for
  wasm32 against genet main `01d3f52`, because `RenderCore::renderer` is native-only there
  (`genet-render-host/src/lib.rs:757`) and the example read it twice. Repaired the same evening at
  Mark's word: both reads go through `cambium_rootstock::try_renderer`, the one-operation checked
  access that exists on every target, and the proof JSON records a refusal (busy frame gate or
  lost device) in place of a value it never read, instead of inventing one. The example's wasm32
  check passes.
- 2026-10-26 (L1, third pass, Mark's rulings applied): Q010, a malformed facet record is a
  `Replay::Malformed` outcome the checked loops refuse and the tolerant loops drop, with obsolete
  imagery an explicit `Skip`; Q031, one rule in `graph/assertion_handles.rs` serves checked and
  recorded load, selection freeze and link freeze with one reason, and the caller-qualified
  migration dedupes the copies it builds and says so in a `LegacyMigrationNote`; Q016's edge
  replay arms consult the widened handle validator (`PendingWrite`) and migration runs inside the
  explicit `stage_edges`/`settle_staged_edges` scope. Kernel lib 505/505 and both reopen tests,
  Pandect 340 lib and 28 integration tests green; one Pandect frozen-record test now asserts the
  refusal of an identical copy. New forks in §5: Q031's legacy baselines that already hold copies,
  and whether migration drops or refuses a malformed facet record. Receipt updated.
