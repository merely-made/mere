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
  Mark, 2026-10-26: **"Dedupe both (Recommended)"**. Whole identical rows dedupe to the first in
  the Surface write-back and in pre-existing Resource rows alike, each with a note on the owning
  Resource; a repeated handle on rows that differ stays a refusal, since that is a conflict, not a
  copy.
  Landed the same evening (`2d5601db5`): whole-row copies dedupe in both strata with a note each.
- **Q010 in legacy migration** (L1, 2026-10-26, the lane stopped rather than picked). Checked
  replay now refuses a malformed facet record, but the caller-qualified legacy migration replays
  its journal under `OnRefusal::Drop`, and `legacy_resource_migration.rs:395` still drops such a
  record with `if let Some(delta) = entry.delta.replay_delta_as(..)`. Either migration refuses it
  as a `LegacyMigrationError`, as `check_typed_record` at `:398` refuses an invalid typed resource
  capture; or it keeps dropping, since the migration is caller-qualified and its diagnostics could
  count the drop. Once decided, the three drop-both adapters in `capture/replay_outcome.rs` go.
  Mark, 2026-10-26: **"Refuse it (Recommended)"**. Migration returns a `LegacyMigrationError`
  naming the node and facet, as it does for an invalid typed resource capture; the three drop-both
  adapters in `capture/replay_outcome.rs` go.
  Landed the same evening (`5936edf2c`): migration refuses the record naming node and facet; the adapters are gone.
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

Open fork from L1's chartulary pass, 2026-10-26 (a question with the code evidence; unnumbered
until Mark rules; the lane repaired Q001, Q003 and Q019 and stopped here rather than picked):

- **Q002, what binds a checkpoint to the journal it summarizes.** Two admission paths check only
  that the checkpoint's sequence does not pass the journal's length. Chartulary's
  `GraphLog::load_checkpointed` (`spine.rs:732`, the check at `:756`) reads a `Snapshot`
  (`spine.rs:70`) whose fields name no journal beyond `at_seq`, which `snapshot` fills from
  `self.log.len()` (`:703`). On eidetic main `af17fe7`, a one-node log `durable` beside a one-node
  snapshot of an unrelated graph `foreign` loads `foreign` at revision 1, where `load_full` gives
  `durable` at revision 1. Pandect does not call that loader (nothing in mere does; kernel
  `graph/journal.rs:25` only mirrors it). `GraphSession` open has its own
  `Checkpoint { cursor, translated_receipt_digest }` (`graph_session.rs:313`), admitted when
  `cursor <= live` (`:774`), then reads `graph.json` and `facets.json` without binding either to
  the journal; only the legacy-translation path carries a digest (`:749`). muniment's `Journal`
  carries a `LogId` and `Provenance` but no content digest (`journal/persist.rs:10` to `:20` lists
  hash-indexed entries as roadmap). So one rule has two owners, and repairing either leaves the
  other open. Either (a) muniment's journal offers a digest of its first n entries and both
  checkpoints store and verify it: one binding, and a new muniment API; (b) each checkpoint owner
  computes its own, so `Snapshot` and `Checkpoint` each gain a field and the format is duplicated
  again (A016); or (c) a checkpoint stops being trusted and is verified by replay, or dropped,
  which gives up the optimization. Whichever binds, a stored checkpoint with no digest is then
  refused, ignored in favour of a full replay (the path legacy assertion-less snapshots already
  take, `spine.rs:740`), or trusted as today.
  Mark, 2026-10-26: **"muniment digest (Recommended)"** and, for a stored checkpoint with no
  digest, **"Replay instead (Recommended)"**. muniment's journal offers a digest of its first n
  entries; chartulary's `Snapshot` and Pandect's `Checkpoint` both store and verify it, one binding
  with one new muniment API; a digest-less checkpoint is ignored in favour of a full replay, the
  path legacy assertion-less snapshots already take, and is re-checkpointed with the digest.

Open forks from the Q002 repair, 2026-10-27 (questions with the code evidence; unnumbered until
Mark rules; the lane applied his two answers and stopped at these rather than picked):

- **Q002, what binds Pandect's `graph.json` and `facets.json` to their checkpoint.** The journal
  digest lives in `checkpoint.json` (`Checkpoint.journal_digest`, verified at
  `graph_session.rs:796`), but the graph and its facets are separate records the open then reads
  without any check against it (`:805`, `:808`), written in the same batch as the checkpoint
  (`:1432` to `:1433`, with the record at `:1442`). So a checkpoint of this journal stays valid
  while `graph.json` and `facets.json` are replaced by another graph's, which is exactly what the
  audit's probe does: `pandect_checkpoint_reason_diagnostics.rs`
  `recorded_session_accepts_foreign_checkpoint_at_its_own_valid_cursor` replaces only those two
  keys and still passes on the repaired code, the foreign node opened as the session's graph. The
  journal digest cannot reach this; chartulary does not have the gap because its `Snapshot` is one
  record carrying contents and digest together (`spine.rs:75` at eidetic `cceb203`). The repaired
  cases are a whole checkpoint (record and files) taken beside another journal, which open then
  refused as Corrupt, and a checkpoint with no digest, which open ignores for a full replay.
  Either (a) the checkpoint
  record also stores a muniment `Hash` of the stored bytes of `graph.json` and `facets.json`, which
  open hashes as read: bytes as stored, no new format for the files, and a hand edit of either
  file then fails the check; (b) the three become one record, so contents, cursor and digest
  cannot be separated: a format change that retires two files other tools may read; or (c) the
  kernel's `GraphSnapshot` and `NodeFacetStore` each carry the digest, a change to two kernel
  formats. And whichever binds, a mismatch is Corrupt, as a foreign journal digest now is, or the
  checkpoint is ignored for the replay, as a missing one is.
  Mark, 2026-10-27 (asked in the Q002 session): **"Hash in the record (Recommended)"**.
  `Checkpoint.contents_digest` holds a muniment `Hash` of the stored bytes of `graph.json` and
  `facets.json`, each length-framed; open hashes the bytes as read, and a checkpoint that names no
  such hash is treated as naming no files. One reading, ruled below: a session with a
  retained legacy translation is not hashed, because its checkpoint is already compared with that
  translation and a difference there is refused as Corrupt (the existing
  `translated_checkpoint_rejects_valid_truth_changes_before_admission`), which his mismatch answer
  below would otherwise have turned into a replay.
  Re-asked in the parent session later the same morning, with the evidence, before that session
  had pulled the landing: Mark answered **"Stored-bytes hashes (Recommended)"** and
  **"Corrupt (Recommended)"**. Shown that the Q002 session had already asked and landed the
  opposite mismatch rule, he ruled **"Replay past stands (Recommended)"**: the mismatch answer
  here is withdrawn and the landed rule stands. On the reading above: **"Keep the refusal
  (Recommended)"**, the translation is a qualification record, not a cache, so a checkpoint that
  differs from it stays Corrupt. Asked again in the Q002 session, which had not yet pulled that
  answer, he replied the same: "Yes: a retained-translation session keeps its comparison with the
  translation and refuses a difference as Corrupt." Ruled; no code changes.
- **Q002, a digest the entries' encoding can move.** The journal digest is taken over each entry's
  canonical serde encoding (`muniment` `journal/digest.rs`), because stored bytes change with the
  codec and, for a hash-ordered map, from one save to the next. So an entry type whose
  serialization changes for existing entries (a field added without `skip_serializing_if`, a
  renamed field) changes the digest of every stored checkpoint, and a refused mismatch then
  rejects a checkpoint that a replay would have read correctly: `GraphLog::load_checkpointed`
  returns `StoreError::Codec` and Pandect open returns `SessionError::Corrupt`, as the existing
  refusal of a checkpoint past the journal does (`spine.rs:758`). Either the refusal stands and an
  entry type's owner versions any change to how existing entries encode; or a mismatch is
  treated as unverifiable and replayed, which keeps old checkpoints usable across such a change
  and gives up the loud signal for a substituted checkpoint.
  Mark, 2026-10-27 (asked in the Q002 session): **"Replay past it (Recommended)"**, and for the
  matchable `StoreError` variant, **"Keep Codec (Recommended)"**, which is moot once a mismatch
  replays. A checkpoint is a cache the journal outranks: chartulary's `load_checkpointed` and
  Pandect's open ignore one that names no journal prefix, another journal's prefix, a prefix past
  the end of this journal, or (Pandect) files that are not the ones it wrote, replay in full, and
  the next checkpoint names what it was written with. No digest mismatch is an error any longer.
  chartulary's refusal of a legacy (assertion-less) snapshot past the journal
  (`legacy snapshot sequence ... exceeds journal length`) is outside this and stays.
  Re-asked in the parent session the same morning: **"Refuse, with a version (Recommended)"**
  (a stored entry format version beside the digest, a different version replays, a same-version
  mismatch Corrupt, golden digest tests pinning each entry type's encoding). Shown the landing, he
  ruled **"Replay past stands (Recommended)"**. No entry format version is stored: an encoding
  change invalidates every stored checkpoint once, silently, and the next checkpoint names the
  new encoding.

### Open forks from L1's owner round, 2026-10-28

The lane's second slice: the five authority entries the first round named and the graphshell host
findings that waited on them (§5.2). Each entry lists what is copied today, with the code at main
`295c39cd8`, and the single owner the lane proposes; Mark's answer follows each.

- **A001, three copies of the navigation mutation.** `graph/mod.rs:772-814` keeps `navigate_node`,
  `branch_history`, `node_history_back` and `node_history_forward` as `pub(crate)` raw wrappers;
  the live arms in `graph/apply.rs:1190-1255` repeat the same mutation and add the capture,
  `refresh_surface_resource` (the Resource record and shown binding a navigation leaves, which
  `tests/resource_lifecycle.rs:198` asserts) and the shared retry tail; the replay arms at
  `apply.rs:1310-1392` repeat it a third time with the captured stamp. No production code calls the
  raw wrappers: they are reached through `GraphFixtures` (`fixtures.rs:113-122`) and four lines of
  `tests/snapshot_navigation.rs`, so a fixture graph navigated through the trait lacks what a live
  navigation leaves, and three copies are kept in step by hand. The `mere` crate's `fixtures`
  feature (`crates/mere/Cargo.toml:61`) is an ordinary feature, so `--all-features` on the root
  enables the hatch outside dev-dependencies. One owner: a `pub(crate)` primitive per navigation
  operation, taking the transition, stamp and session, under both arms; the raw wrappers go, the
  Mark, 2026-10-28: **"Delete; one primitive (Recommended)"**. The raw wrappers go, one
  `pub(crate)` primitive per navigation operation serves both arms, fixtures and tests navigate
  through the delta, and the root crate's ordinary `fixtures` feature goes with them.
- **A003, two recency clocks.** Stemma's visit clock is written by every navigation and read by
  `recent_visited` (`graph/history.rs:333`), whose doc claims one shared visit space and "no
  separate recency list" and which no production code calls. The facet `visit.history`
  (`node_facets.rs:33`) `last_visited_ms` is written at node creation (`mod.rs:654`) and by
  `TouchNodeLastVisited`, the canvas revisit of an existing URL (`pictograph/canvas/input.rs:573`),
  and read by the canvas recency channel (`signals/producers.rs:243`) and the `last_traversal`
  filter key. `NavigateNode` (`apply.rs:1190`), which the in-place navigation at
  `pictograph/canvas/nodes.rs:308` sends, and the history steps write Stemma only, so navigating
  a node to a new page does not move its recency in the canvas while revisiting its URL does. Each
  Stemma read rebuilds the whole memory (`history.rs:164`), so recency over every node cannot be a
  Stemma projection, and a touch is not a navigation (it would add a no-op step to back and
  forward). One owner: the facet holds recency, stamped by one primitive from `NavigateNode`, the
  history steps and the touch, with the replays carrying the stamp; Stemma holds the trail and
  Mark, 2026-10-28: **"Facet owns recency (Recommended)"**. One primitive stamps `visit.history`
  from `NavigateNode`, the history steps and the touch; replays carry the stamp; `recent_visited`
  and its claim go; Stemma is the trail.
- **A003, the session clock has no writer.** `last_session_visited` is stamped from
  `Graph::current_session`, which only `set_current_session` sets; its one non-test caller is a
  pictograph wrapper (`canvas/lifecycle.rs:330`) nothing calls, so every node carries session 0.
  `PersonaSettings.session_count` (`persona_settings_store.rs:66`) exists to count launches and is
  read only by its test. Alembic's `KeepSessions` eviction (`memory_levels.rs:132`) treats 0 as
  never stale and so never evicts, and `KeepDays` reads a per-node map no host builds. Either the
  host that opens a persona's session installs the launch counter, with a regression that a
  navigation in launch N stamps N; or the session clock, `session_count` and `KeepSessions` go,
  Mark, 2026-10-28: **"Wire the launch counter (Recommended)"**. The host that opens a persona's
  session increments and persists `session_count` and installs it with `set_current_session`; a
  navigation in launch N stamps N and `KeepSessions` evicts by it.
- **A012, who decides a graph is loadable or saveable.** The kernel's checked load
  (`snapshot/checked.rs:178`: navigation indices, resource columns, the one handle rule) is the
  structural validator, run by every load through `materialize_snapshot` and by addressable
  publication (`addressable_graph/write.rs:58`). Saves disagree: the file saves in
  `session_graph_store.rs` validate nothing and have no caller left (`GRAPH_FILE` is the module's
  only use); the unchecked codicil saves (`graph_codicil.rs:294-372`) validate nothing; the
  profiled codicil save runs the unprofiled loader whatever the placement (`:383`); the checked
  archive save materializes and runs the product closure (`:517`); session checkpoints
  (`graph_session.rs:1453`) run only the optional product validator and rely on the write path.
  Facets are installed by `*graph.facets_mut() = ...` with the post-overlay handle check
  (`resource_content.rs:177`) a separate call at six Pandect and six host sites, and three loaders
  skip it (`mere-overmap/wire.rs:67`, kernel `residency.rs:18`, `addressable_graph/write.rs:58`).
  Checked load keeps the first of two Surfaces with one id (`from.rs:89`) and drops an unparseable
  id (`:73`) without saying so, which Q010's ruling refuses for facet records. One owner: the
  kernel's checked load decides structure and refuses a repeated or unparseable Surface id as it
  refuses a malformed facet; facets install through one checked primitive that runs the handle
  check; every archive save runs the checked loader with its own placement; product validators
  stay the product's, after structure; the dead `session_graph_store` save and load go (A004 to
  two paths).
  Mark, 2026-10-28: **"Checked load decides (Recommended)"**, as proposed.
- **A016, one canonical form or one per comparison.** Comparisons of graph truth each
  canonicalize differently: `replay_state` (`graph_session.rs:636`) compares snapshot and facets
  as `serde_json::Value` in `to_snapshot_at` order; `FrozenGraph::of` (`graph_placement.rs:114`)
  sorts Surface edges only; the residency digest (`addressable_graph/residency.rs:68`) sorts
  fields and couplings by id; the addressable record hash binds each edge's position in the whole
  snapshot (`write.rs:86`); pictograph (`canvas/lifecycle.rs:871`) and `product.rs:834` sort on
  their own. The kernel still holds `fields` and `couplings` in `HashMap`s (`graph/mod.rs:379`)
  emitted in iteration order (`snapshot/to.rs:119`), so Q034's false refusal stands; Stemma's
  snapshot is sorted since Q019 except `children` order. Byte digests (`source_digest`, the
  translation digest, Pandect's `contents_digest`) bind stored bytes, a different obligation. One
  owner: the kernel's snapshot is canonical at the source (ordered maps for fields and couplings,
  edges and resource edges in identity order, Stemma's sorted snapshot), so every comparison and
  content hash sees one order and the per-site sorts go; byte digests over stored files stay.
  Mark, 2026-10-28: **"Canonical at the source (Recommended)"**, as proposed.
- **A017, what counts as an identity reference in a pick merge.** `remap_edit`
  (`graph_session_merge.rs:244`) serializes a `CapturedDelta` to JSON and rewrites every string
  and key equal to a mapped id, so a title, tag, URL or body equal to a UUID is rewritten (Q020,
  whose audit probe still passes against main), and ids inside a facet's `value_json` string are
  never rewritten. The kernel already enumerates every id-bearing field per variant in
  `Touched::add` (`graph/revert.rs:106-186`, "exhaustive on purpose"), which the merge uses only
  to filter. The other copy paths decide identity through typed fields with policies of their own:
  `copy_component_from` (`cross_graph.rs:144`), `merge_snapshots_with_remap` (`snapshot_merge.rs:193`),
  `copy_node_facets` (`facet_store.rs:101`), `import_edits` (`merge.rs:40`). One owner: a typed
  remap on `CapturedDelta`, exhaustive by variant beside `Touched`, with facet values remapped
  through the facet's own declared id fields and other facets carried verbatim; the other copy
  paths are recorded as separate operations with their own rule.
  Mark, 2026-10-28: **"Typed remap by variant (Recommended)"**, as proposed.
- **Q056, a peer told Accept for a session the host refuses.** `admit_session`
  (`graphshell-host/src/admission.rs:168`) keeps Notochord's Accept reply and turns the outcome
  into `ActionNotOffered` when the owner allowed an action Graphshell does not serve; `host_join`
  (`webrtc_session.rs:245`) writes that reply before testing the outcome, and the peer maps
  Accept to a completed join (`webrtc_join.rs:310`), then sees the channel close. The native
  carrier documents the same two stages on purpose (`carrier.rs:125-135`: "the handshake did not
  deny it: this service did"), and `webrtc_session.rs:148` says every refusing path writes the
  refusal, which this path does not. No crate test checks reply and outcome together for this
  case; the audit's probe does. Either the host's vocabulary is part of what admission answers
  and `ActionNotOffered` is written as the reply on every carrier before anything closes; or the
  two stages stay and the peer learns to treat an Accept followed by a close as not served.
  Mark, 2026-10-28: **"Write the refusal (Recommended)"**. The served vocabulary is part of what
  admission answers: `ActionNotOffered` is written as the reply on every carrier before anything
  closes, the native carrier writes it before shutdown, and a test checks reply and outcome
  together.
- **Q057, a reconnect claim admission does not enforce.** `lifecycle.rs:22-29` says a reconnect
  "produces a different id ... by construction"; `projection_session`'s own doc (`:57-64`) says
  nothing checks that a nonce is fresh, and the test (`:471`) picks two different ids by hand. The
  id is Notochord's transcript digest over the client nonce, subject, transport identity and
  shared link; the WebRTC carrier mixes a fresh host nonce into the link on every join
  (`webrtc_session.rs:183`), so there the claim holds through the carrier, and on a carrier
  without one two admissions of one hello give one id (the audit's probe). Nothing stores seen
  nonces. Either the prose and the test claim what admission enforces, freshness from the
  carrier's link or the client, with a test that admits one hello twice; or the responder supplies
  freshness on every carrier, which is a Notochord change.
  Mark, 2026-10-28: **"Claim what is enforced (Recommended)"**. The prose and the test say
  freshness comes from the carrier's link or the client; the test admits one hello twice on one
  link and once on a fresh link; no replay state.

### Open forks from L1's remainder, 2026-10-28 (the second owner round)

The rest of the lane in seven file sets, read against main `042843343`: the §3 cues that point at
L1, the kernel's address and write-boundary entries, and the pictograph and web findings whose
repair needs an owner choice (the rest of those two crates' findings sit with L2, L5 and L6, listed
in the receipt). Each entry names what is copied today and the single owner the lane proposes;
Mark's answer follows each.

- **Q253, a Forme save that compares nothing.** The audit's `PreparedSave::save` is gone; the
  blind write moved to the host: `web_tree/forme.rs:112-115` puts the bytes with `backend.put`
  and the session never holds the bytes it loaded (`forme.rs:40-44`, `:323-325`). The rules editor
  refuses a stale write inside `backend.transact` by comparing the held bytes with what it loaded
  (`presentation_store.rs:150-157`), and `block_editing/store.rs:67-116` is a third copy of the
  same compare. Every guard Forme has is in-process (request token, busy, write fence, load gate),
  and `forme_session_lifecycle_tests.rs:735` asserts the old proposal's blind put as fine. One
  owner: one compare-and-replace over a slot, on muniment's `Backend::transact`, used by the rules
  store, the block store and Forme's pane, which keeps its loaded baseline and keeps the draft on a
  conflict as `finish_save(Err)` already does.
  Mark, 2026-10-28: **"One slot guard, three users (Recommended)"**, as proposed.
- **Q214, three writers of one settings file.** `pairing.rs:75-88` (pair), `:105-112` (unpair)
  and `personal_sync.rs:939-952` (the resident's dial hint) each load, modify and save
  `OwnerSettings` with no revision check or lock between; `pairing.rs:46-48` still says pair and
  unpair are the only writers and two processes never race. `OwnerSettings::save` funnels into
  `write_bytes_with_backup`, an atomic replace that protects no earlier read. The muniment
  directory lock is single-owner and long-lived, which is why management verbs avoid it. One
  owner: `OwnerSettings` gains one load-modify-commit that writes only if the file still holds the
  bytes it loaded, under a short advisory lock around the compare and the rename, refusing a
  conflict so the caller reloads and reapplies; all three writers use it, the claim is corrected.
  Mark, 2026-10-28: **"Load-modify-commit (Recommended)"**, as proposed.
- **Q061, a refused second request overwrites the first.** `SessionCore::begin` (`core.rs:726`)
  sets `pending` unconditionally; `SessionDriver::discover` (`driver.rs:126-134`) calls
  `rediscover` before `send` checks `inflight` (`:217-222`), so the refused second start leaves
  the core expecting a descriptor and the original Closed answer is then refused. `submit_command`
  alone guards with `is_awaiting` (`core.rs:413`); the other nine starters and `core_mut` do not.
  The test at `driver.rs:394` asserts only the refusal, never the original's answer. One owner:
  `begin` refuses while an operation is pending and every starter returns that refusal, so the
  driver's inflight and the core's pending cannot disagree; the internal follow-ups keep taking
  the pending first.
  Mark, 2026-10-28: **"The core refuses while pending (Recommended)"**, as proposed.
- **Q240 with A550 and A551, a rejected capture leaves graph edits pending.** `capture.rs:685`
  and `:740` edit the host through `edit_now` before anything is stored, and on a staging or
  commit failure `:793-803` restore only `self.memory`; the ledger put, access manifests and trace
  writes in the batch are dropped while the graph edits stay pending in `MereHost` and the next
  `stage` or `persist` writes them without their records (`forget_url` has the same shape,
  `:824-884`). The control, `transact_change` (`graph_session_transaction.rs:143-280`), builds a
  detached candidate, commits in one `backend.transact` with rechecks and installs on success, but
  it is crate-private, takes no extra write ops or keys, rechecks only the session's own keys and
  refuses while edits are unstored. One owner: a transaction entry the host can reach that takes
  the capture's graph change as deltas on the candidate plus its ledger, manifest and trace ops
  and the keys they depend on, commits once and installs; capture edits nothing live before that
  and restores its in-memory browsing memory on refusal as today.
  Mark, 2026-10-28: **"One candidate, one commit (Recommended)"**, as proposed.
- **A004, the flush path rechecks nothing.** With the kernel store (Q017) and the file saves
  (2026-10-28) gone, `GraphSession` writes by `flush` (`graph_session.rs:1581-1599`, plain puts of
  journal and change entries at `[saved, len)`), by the compare-and-set `transact_change`, and by
  checkpoint; the archive codicil saves and the addressable publication stand beside it. Two
  handles opened at one `saved` that both edit and flush overwrite each other's entries, last
  writer wins; the compare-and-set path refuses the same case ("Session changed before commit").
  No test exercises two flushing handles. One owner: `flush` commits under the same rechecks as
  `transact_change` (log lengths and tails), on a backend that transacts, so a stale handle is
  refused rather than overwriting.
  Mark, 2026-10-28: **"Flush rechecks like transact (Recommended)"**, as proposed.
- **Q257 with A067, a save acknowledgement that names nothing.** The projection's save keeps one
  Boolean (`web_tree/product/projection.rs:102`, set at `:677`); the host's generic completion
  (`product.rs:1384-1388`) then calls `saved()` (`projection.rs:371-382`), which marks whatever
  editor is current clean. `load_projection_input` (`:385-426`) replaces the editor on a dataset
  change from the frame hook (`web_tree.rs:2014-2030`) without touching the pending flag, so a
  replacement draft is marked clean by the old save. Link visibility already binds its
  acknowledgement to an identity (`product.rs:1257-1308`). A067: chirograph's doc still calls the
  wire acknowledgement "the last revision a client has durably applied" (`chirograph/src/lib.rs:578`)
  while it carries in-memory scene progress. One owner: the pending save records what it sent (the
  dataset identity and editor generation) and completion marks clean only that; a replacement
  during a save stays dirty; the chirograph doc says applied in memory.
  Mark, 2026-10-28: **"The ack names what it saved (Recommended)"**, as proposed.
- **Q243 and Q244, the extension's capture storage has two rules.** `background.js:212-225`
  (configure) and `:291-294` (permission removed) write the queue, visit map and policy outside
  `serializedQueueMutation` (`:64`), which the queue and ack writers use, so a configure can drop a
  concurrently queued visit. `capture-model.js:207-211` refuses a conflicting visit on url, time,
  transition and parent, while the ledger's `same_source_facts` (eidetic
  `import/encounters.rs:68-85`) also compares `original_unix_micros` and `source_is_local`, and
  a thrown conflict aborts the whole batch. One owner: every capture-storage write goes through
  the one lock, and the queue's conflict rule is the ledger's, checked by a fixture generated from
  the Rust rule, refusing the one visit rather than the batch.
  Mark, 2026-10-28: **"One lock, the ledger's rule (Recommended)"**, as proposed.
- **The capture acknowledgement has no sender.** `extension-profile.js:246` listens for
  `graphshell-capture-complete` and the extension README says the host acknowledges visits after
  the batch persists, but nothing dispatches it since `93ce49c82` removed `publish_capture_receipt`;
  the queue is never acked and every redelivery is absorbed by the ledger's duplicate drops
  (`capture.rs:677-679`). Either the host dispatches the acknowledgement after `staged()` and the
  queue drains; or the listener and the claim go and redelivery is the documented contract.
  Mark, 2026-10-28: **"Restore the ack (Recommended)"**. The host dispatches it after `staged()`
  for the visits the batch carried; a persisted batch is acked, a refused one is not.
- **Q247, the local-file exclusion strips a facet and keeps the address.** Export's
  `filtered_facets` (`product.rs:1042`) drops the local-file facet when the box is unchecked, and
  `transfer.rs:345` forces it off, but `filtered_snapshot` keeps the node's `file:` address in
  both address fields and the access-history facet keeps `address` and `referring_address`; every
  test uses an `ni:` address, so none exports a `file:` one. One owner: the exclusion is decided
  by the typed address (`Address::File`), applied to the node's addresses, the legacy url field and
  the access-history records alike, with a test that exports a `file:` address.
  Mark, 2026-10-28: **"Typed address decides (Recommended)"**, as proposed.
- **A011, the address and the history cursor can disagree.** Primary is written by
  `update_node_url` from `SetNodeUrl` (`apply.rs:756`), from `navigate_node` after the Stemma
  visit (`mod.rs:788-802`) and from a history step (`land_history_step`, `mod.rs:840-843`); load
  keeps primary since Q013. `NavigateNode{A}` then `SetNodeUrl{B}` leaves primary B and cursor A,
  and the next step re-imposes the cursor's page on primary, discarding the set address;
  `node_current_url` (`mod.rs:860`) reads the cursor and its one caller checks only existence. No
  test asserts the two agree. One owner: the primary address is the node's address and the Stemma
  cursor is history, said in the kernel's docs; `node_current_url` goes, and a history step lands
  on primary as it does, which is the one place history writes the address.
  Mark, 2026-10-28: **"Primary owns, cursor is history (Recommended)"**, as proposed.
- **A028, what counts as the write path.** The kernel comment names the public assertion and
  declaration writers (`mod.rs:580-606`); `GraphSession::edit_now`'s doc still says every mutation
  goes through `apply_graph_delta`, "the kernel's one write path" (`graph_session.rs:1333-1336`).
  In fact `edit_as` captures any writer's emitted deltas under a recorder, replays them onto a
  fresh clone and refuses a candidate that differs (`:1346-1399`), so the session's
  capture-and-replay check is the boundary and `GraphDelta` is one of its writers; the
  compare-and-set wrappers accept deltas only and the generic `transact_change` is crate-private,
  so djinn converts direct writes to deltas first (`knot_relation_intake.rs:295-343`). One owner:
  the docs name the capture-and-replay check as the boundary, and `transact_change` opens to hosts
  so a direct write transacts under the same check.
  Mark, 2026-10-28: **"Capture-and-replay is the boundary (Recommended)"**, as proposed.
- **Q100 and Q101 with Q091, who reads a node's neighbours and statements.** Pair Unrelate
  (`selection.rs:333-344`) takes the first Surface arc from `find_edge_key` and its sub-kinds, so
  on a Resource-stratum relation (Cites, Hyperlink, everything but UserGrouped) it removes nothing
  and reports 0, while the kernel's `retract_relations` already withdraws across both strata
  (`edge_ops.rs:775-877`). `isolate_selection` (`view.rs:155-160`) and degree sizing
  (`cartography.rs:466, 591`) read the Surface-only `neighbors_undirected`; the canvas holds six
  neighbour readers with different hide and self-loop rules (`signals.rs:69`, `producers.rs:24`,
  `fold_projection.rs:205`, `seiche_bridge.rs:53`). One owner: the kernel enumerates, both strata,
  through one neighbour reader the canvas readers adopt (hides applied by the visible-row reader
  only), and Unrelate asks the kernel's `retract_relations` for the pair.
  Mark, 2026-10-28: **"The kernel enumerates (Recommended)"**, as proposed.
- **Q100, whose statements a canvas gesture may withdraw.** The kernel's
  `withdraw_semantic_assertion(id, None, None)` checks no ownership; graphshell's
  `promoted_links::can_withdraw` (`promoted_links.rs:300-340`) requires the current persona as
  asserter, user scope and no method. Either the gesture withdraws only what the host's rule
  allows, asking the host; or the canvas withdraws any statement on the pair.
  Mark, 2026-10-28: **"What the host's rule allows (Recommended)"**. The gesture asks the host,
  which applies `can_withdraw`; imported or method-backed statements stay and the gesture reports
  what it left.
- **Q209, two admission computations for suggested links.** `refresh_link_suggestions` takes a
  mint budget from counts of its own before minting (`link_suggestions.rs:137-145`) and
  `admit_suggestions` recomputes admission afterwards from a different candidate set
  (`suggestions.rs:59-71`); the source gate applies to the links source only. One owner: one
  admitted set, computed once after source filtering, from which minting and attention both read.
  Mark, 2026-10-28: **"One admitted set (Recommended)"**, as proposed.
- **Q210, which page is a kept link's origin.** The first eligible page per URL wins
  (`link_suggestions.rs:149-162`), the origin is written once into the derived tuple (`:200`) and
  never updated, and the graph relation page to member is a Hyperlink the host later writes
  durably from that tuple; a withdrawn page stays the origin of a shared URL and `create_link`
  mints an unlinked address when it is gone. Member identity comes from three sources (the FNV
  `link_member`, the kernel's insert-if-missing, the host's held-by-URL or fresh id). One owner:
  the graph relation holds the origin and the kept link reads it at keep time; the offers list
  only dedupes.
  Mark, 2026-10-28: **"The relation holds the origin (Recommended)"**, as proposed.

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
  `ports/mere-resident/tests/residue_verdict`, unjudged host blocks printed with their reason, regression
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
- 2026-10-26 (L1, chartulary): Q001, Q003 and Q019 repaired on eidetic main (`af17fe7`; branch
  `l1-chartulary` merged and pruned) and mere repinned to it, in the workspace and in
  `ports/graphshell/web`, lockfiles changed in their source revs only. Q001 follows Mark's creator
  ruling (re-parent to the deleted owner's creator, else clear) and also clears a survivor's pending
  origin parent that the deletion collected; Q019 writes snapshot bindings and owned visits in index
  order; Q003 archives a shared nested graph only when its last bearer leaves. One regression each
  in chartulary (three for Q001 and Q019, one for Q003 beside a control) and a consumer before/after
  in the kernel and Pandect; the audit's probes now fail on their defect assertions. Q002 stays open
  as a fork (§5): its binding has two admission owners. Chartulary 87/87 (79 before), kernel lib
  485/485 on its own base and 506/506 merged with the third pass, Pandect lib 342/342. Receipt:
  `support/receipts/ownership-repair/l1/2026-10-26_l1_chartulary_receipt.json`.
- 2026-10-26 (L1, fourth pass, the migration rulings): legacy migration dedupes whole identical
  rows in both strata with a `LegacyMigrationNote` each (`legacy_resource_migration/copies.rs`)
  and refuses a malformed facet record as a `LegacyMigrationError` naming node and facet; the
  drop-both replay adapters and the pub `CapturedDelta::replay_delta` / `replay_delta_as` are gone
  (a public-API change with no caller left). Merged onto the chartulary repin. Kernel lib 512/512
  with both reopen tests, Pandect 342 lib and 28 integration tests. Of L1's first slice only Q002
  (chartulary, a fork in §5) and Q014 (re-document only) remain. Receipt updated.
- 2026-10-27 (L1, Q002 applied): muniment gains `Journal::prefix_digest(len)`, a blake3 digest of
  the first `len` entries over their canonical encoding (maps ordered by key; a prefix keeps its
  digest as the log grows, across saves and in forks). Chartulary's `Snapshot` stores it,
  `load_checkpointed` refuses a snapshot of another prefix and replays in full past one with no
  digest (Mark's two answers); `Container` tags serialize sorted so a journal entry holding them
  digests alike after a reload. Pandect's `Checkpoint` stores it, open refuses a mismatch as
  Corrupt naming the checkpoint (replayed past since, below) and ignores a digest-less checkpoint
  for the baseline-plus-journal replay, and the next checkpoint writes the digest. Regressions in muniment (7), chartulary (6)
  and Pandect (3); mutation controls that remove the check, the digest-less path and the sorting
  each fail the matching ones. The audit's swapped
  `graph.json`/`facets.json` probe still opens the foreign graph, because the journal digest cannot
  reach files outside the record that holds it: that, and what an entry-encoding change does to a
  stored digest, are two forks in §5. Eidetic `cceb203`; kernel lib 513, Pandect lib 347 and its 28
  integration tests (other sessions' tests are in those counts). Receipt updated.
- 2026-10-27 (L1, Q002 follow-through, Mark's two answers applied): a checkpoint that cannot be shown
  to match is replayed past instead of refused, in chartulary (eidetic `e7eef1e`) and Pandect, and
  Pandect's checkpoint record now also hashes the stored bytes of `graph.json` and `facets.json`, so
  the audit's swapped-file probe no longer opens the foreign graph (the probe now fails at its
  defect assertion). The earlier foreign-journal regression changed from Corrupt to replay; two
  regressions are new, one for replaced files and one for a record that names no files. A session
  with a retained translation keeps its own comparison and refusal (ruled by Mark, §5). Kernel lib 513, Pandect lib 349 and its 28 integration tests, chartulary 94, muniment 54.
  `stickleback` (105 and 5 tests) passes on eidetic `cceb203`; the full-workspace stall was feature
  unification, not that crate. Receipt updated.
- 2026-10-27 (L1, the Q002 forks asked twice): the parent session walked Mark through both
  forks with the evidence an hour after the Q002 session had asked and landed them, without
  having pulled main first, and got the opposite mismatch answer. Shown the conflict, Mark ruled
  the landed rule stands (replay past) and confirmed the translated-session refusal; both are
  recorded under the forks in §5. Nothing in code changed. A fork is answered in §5 before it is
  asked again: pull main and read §5 first.
- 2026-10-28 (L1, the owner round): the five authority entries the first round named (A001, A003,
  A012, A016, A017) and the graphshell host findings that waited on them (Q056, Q057) walked
  through with the code at `295c39cd8`; eight forks recorded in §5 with Mark's answers the same
  morning: one navigation primitive under both arms with the raw wrappers and fixture methods
  gone; the facet owns recency and Stemma the trail; the launch counter is wired into the session
  clock; the kernel's checked load decides structure, with one checked facet install and the dead
  `session_graph_store` saves gone; the kernel's snapshot is canonical at the source; a typed remap
  by variant replaces the JSON walker; `ActionNotOffered` is written as the reply on every
  carrier; the reconnect claim says what admission enforces. Nothing in code changed. Findings
  not asked, carried as cues: `hydrate()` rebuilds the whole navigation memory on every read and
  mutation (`history.rs:164`); `merge_rules.rs:180` re-decides handle legality for composition
  instead of calling the kernel's one rule; `KeepDays` eviction reads a map no host builds.
- 2026-10-28 (L1, the owner rulings applied): all eight landed on main the same day in two waves of
  worktree agents, one regression each. Kernel: one navigation primitive under the live and replay
  arms with the raw wrappers, the fixtures trait's navigation methods and the root crate's
  ordinary `fixtures` feature gone (A001); the `visit.history` facet owns recency, stamped by
  navigation and the history steps, with `recent_visited` and its claim gone (A003); checked load
  refuses a repeated or unparseable Surface id and `install_facets`/`overlay_facets` are the one
  checked install, `facets_mut` now crate-private (A012); fields and couplings in id order and
  Surface edges in byte order at the source, the per-site sorts gone (A016); `CapturedDelta::remap_ids`
  by variant replaces the JSON walker, with `arrangement.frame-layout` the one facet declaring ids
  (A017). Pandect: every loader installs through the checked primitive, the unchecked and profiled
  codicil saves run the checked loader with their own placement, the dead `session_graph_store`
  saves are deleted. Hosts: `ActionNotOffered` is written as the reply on every carrier (Q056); the
  reconnect prose and test claim what admission enforces (Q057); `record_launch` counts launches in
  the persona settings slot and djinn installs the number on every host it opens, the launch
  surviving a replacement session and qualification (A003 session clock). The audit's Q020, Q034
  and Q056 probes now fail at their defect assertions. Kernel lib 520, Pandect 353 and 28,
  graphshell 313, graphshell-host 104 and 6, pictograph 654, alembic 8; djinn, cambium, overmap,
  rdf, forme, cartography, athanor and mere check. Receipt updated. Cues: Resource records and a
  pair's parallel rows still emit in insertion order; graphshell's web product has no persona boot
  and installs no launch; Notochord exports no reply encoder, so the host refuses by re-judging
  against an action-less policy; `fork_component_graph` changed signature for Turnstone's caller.
- 2026-10-28 (L1, the second owner round): the remainder of the lane read against main
  `042843343` in seven file sets, by four read-only mapping agents whose citations the lane
  verified line by line; fifteen forks recorded in §5 with Mark's answers the same day, each the
  proposed owner: one slot guard for Forme, the rules store and the block store (Q253); one
  load-modify-commit for djinn's settings file (Q214); the client core refuses while pending
  (Q061); one candidate and one commit for a capture (Q240); flush rechecks like the transaction
  (A004); a save acknowledgement that names what it saved (Q257, A067); one lock and the ledger's
  rule for the extension's queue (Q243, Q244) and the capture acknowledgement restored; the typed
  address decides the local-file exclusion (Q247); primary owns the address and the cursor is
  history (A011); capture-and-replay is the write boundary (A028); the kernel enumerates
  neighbours and statements for the canvas (Q100, Q101, Q091), the gesture withdraws what the
  host's rule allows (Q100); one admitted set for suggested links (Q209) and the relation holds a
  kept link's origin (Q210). Q014 closed as re-documented. The other pictograph and web findings
  are placed with L2, L5, L6 or no lane in the receipt. Nothing in code changed.
- 2026-10-28 (L1, the second owner round applied): all fifteen rulings landed on main the same
  evening, by crate in six parallel worktrees with one regression each, in nine merges from
  `267f91b40` (client, Q061) to `09ac76cc9` (transaction: A028, A004, Q240), with a parent wiring
  commit `015f38651` for the canvas withdraw policy, the host-graph adoption seam and the optional
  kept-link origin. Gates on main `09ac76cc9`: kernel 523+3, Pandect 368+28, graphshell lib 320,
  graphshell-host all features 104+6, pictograph 661 (at 015f38651; unchanged since), alembic recall
  8 (at 7a93dc9db; unchanged since); checks graphshell all targets with web, alembic-workshop,
  workspace-editor and editable-blocks; djinn. The web tree is wasm-only and its `graphshell-web`
  check failed inside `spareval` until 2026-10-29: the web lock had split spareval's digest crates
  across two majors (sha2 0.10, sha1 and md-5 0.11); with spareval's sha2 edge at 0.11 (`9dab9408a`)
  the wasm32 check compiles the Forme pane, projection page, product call sites, history and capture
  edits clean. The compiler-cache budget refused every gate for two hours until the four finished
  lanes' caches were retired on Mark's word. Receipt: the operations dated 2026-10-28 from Q061 on
  in `support/receipts/ownership-repair/l1/2026-10-26_l1_receipt.json`.
