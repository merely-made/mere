# Key lane plan

**Status:** 2026-11-06: plan. Nothing has moved. D1 to D3 settle where prekeys travel, which
events stay in the graph log, and how old logs are treated. F1 to F3 are open.

**For:** moving the personal graph's key-agreement carriage off the graph log and onto Eidetic's
`GroupKeyLane` and Murm's `GroupKeySync`, as resident-split D7 ruled ("Plan adoption"). Roster
reconciliation stays in Mere, and this plan owns the wire change.
Not here: the rest of the resident split (`2026-11-05_resident_split_plan.md`); pairing
semantics (`2026-10-28_device_pairing_plan.md`); content sealing, which keeps `DataKeyring` and
`PersonalEncryption::GroupV1` unchanged.

Receipts go to `archive_docs/receipts/2026-11-06_key_lane_plan_receipts.md` (DOC_POLICY §8).

## Constraints

- **Security code follows standards.** Key agreement stays p2panda-encryption (DCGKA) behind
  stickleback's `GroupSession`. This plan moves bytes between carriers; it writes no
  cryptography.
- **Graph keys stay Mere's authority** (resident split, "Mere keeps graph authority").
  `GraphKeyGroup` stays in `mere-resident`; it drives the lane instead of authoring graph events.
- **One record shape per carrier.** New key material never appears in the graph log after the
  cutover, and the lane never carries roster facts.

## Findings (2026-11-06)

Read by a subagent and spot-checked. Paths are under `ports/` in Mere; the Graphshell code is
being extracted to its own repository, so items are also named.

- Mere and the lane use the **same types and library**. `GraphKeyGroup`
  (`mere-resident/src/sync/graph_keys.rs:71`) wraps stickleback's `GroupSession`,
  `GroupPrekeyBundle` and `GroupSessionDispatch`, the types `GroupKeyLane`
  (Eidetic `stickleback/src/group_lane.rs:409`) consumes. Only the carriage differs.
- **Mere's carriage.** `PersonalGraphEvent::PublishPrekey` and `GroupDispatch`
  (`graphshell/src/personal_sync.rs:195`, `:229`) ride the personal-graph log in plaintext,
  strict CBOR, unversioned enum, one kind per operation (`:718`). `key_agreement()` (`:1256`)
  orders them by (seq, hash), not causally. `key_paired_devices`
  (`mere-resident/src/sync/personal_sync_host.rs:879`) absorbs them, then reconciles the session
  toward the folded reader set.
- **The lane carries dispatches only.** It has no prekey record: Murm's test registers prekeys by
  hand (`murm/native/crates/sync/tests/group_key.rs:25`), and a dispatch whose author has no
  registered prekey stays pending (`stickleback/src/group_lane/operations.rs:225`). It applies
  frames in causal order with parking and settled marks, signs under its own group-scoped writer
  salt, and has no roster: it admits any validly attested author for the group.
- **Prekey readers in Mere:** `PrekeyDirectory` (`personal_sync_host.rs:1440`),
  `recipient_for_root` / `root_for_recipient` (`graph_keys.rs:323`, `:338`), `sealing_roots`
  (`personal_sync_host.rs:1386`), pairing facts' embedded bundle (`pairing.rs:173`), publish on
  open (`personal_sync_host.rs:402`) and the relay for receive-only devices
  (`mere-resident/src/personal_sync.rs:690`).
- `GraphKeyGroup` imports Graphshell's `KeyAgreementEvent`, `KeyAgreementStep`,
  `PersonalGraphEvent` and `SyncRoster` (`graph_keys.rs:30`); the Graphshell extraction moves
  those types.
- Mere pins Eidetic `98f2c69`; the lane files are unchanged between it and Eidetic's main.

## Slices

### K1. Prekeys on the lane (Eidetic, D1)
`GroupKeyRecord` gains a prekey kind, so the record version goes to 2. A prekey record's bundle
subject must match the record's attested author root, or the record is refused. Intake registers
the bundle into the session through `drain_ready`, which unblocks dispatches pending on it.
- Done when: the lane tests cover prekey-then-dispatch, dispatch-before-prekey (pending, then
  applied), a bundle whose subject differs from its author (refused) and replay; and the existing
  lane tests pass at version 2.

### K2. Prekey publish over Murm (Murm)
`GroupKeySync` gains `publish_prekey`, and the two-peer test drops its hand registration.
- Done when: `admitted_group_keys_reconcile_rotate_and_release_their_sessions` passes with
  prekeys exchanged over the live transport alone.

### K3. Mere drives the lane (D2, D3)
`GraphKeyGroup` joins one lane per graph (`GroupSessionId` stays the graph id), publishes its
prekey on open, authors dispatches through the lane, and absorbs through `drain_ready` with
persist-then-commit, replacing `checkpoint`/`restore`. Prekey readers read the lane's registered
bundles. `PublishPrekey` and `GroupDispatch` leave `PersonalGraphEvent`; `AdmitReader` and
`RetireReader` stay, and `is_key_agreement` narrows to them.
- Done when: the host tests (`a_paired_device_is_keyed_over_live_sync_and_reads_what_was_sealed`,
  `a_device_keyed_after_sealed_writes_still_receives_them`,
  `a_revocation_on_one_device_reaches_the_others` and their siblings) pass over the lane;
  `the_reader_set_is_a_fold_every_device_computes_the_same_way` passes unchanged; no graph-log
  operation carries a bundle or dispatch; F1 and F2 are settled.

### K4. Clean break (D3)
Graphs keyed before K3 are not carried forward. The resident refuses a graph log containing the
removed variants with a named error, not a decode panic.
- Done when: F3 is settled and a test opens a pre-K3 log and observes that outcome.

## Decisions

- **D1. Prekeys travel on the lane (Mark, 2026-11-06):** "Add prekeys to the lane", over keeping
  `PublishPrekey` as a graph event and over a separate prekey lane. Put to Mark on finding the
  lane carries dispatches only and Mere's prekey readers all read the graph log. Outcome: refines
  resident-split D7, which assumed the lanes covered all key agreement.

  Links: summary Prekey bundles move onto GroupKeyLane as a version-2 record kind; all key material leaves the graph log.

- **D2. Reader-set intents stay in the graph log (Mark, 2026-11-06):** "Stay in the graph log",
  over moving them with the lane. `AdmitReader` and `RetireReader` are roster facts every device
  folds the same way, and the lane has no roster. Outcome: applies resident-split D7 (roster
  reconciliation stays in Mere).

  Links: summary AdmitReader and RetireReader remain personal-graph events gated by SyncRoster; only cryptographic carriage moves.

- **D3. Clean break for old logs (Mark, 2026-11-06):** "Clean break", over keeping a
  decode-and-absorb path for the old variants. No personal graphs on real devices need their
  pre-move history kept readable; DOC_POLICY §3 allows no shim without real user data.
  Outcome: new.

  Links: summary The PublishPrekey and GroupDispatch variants are removed; graphs keyed before the move are not carried forward.

## Forks

- **F1. Receive-only relay.** The resident relays prekeys for devices that cannot author. On the
  lane, a record's author is the relayer, not the bundle's subject, which K1's subject check
  refuses. Options: a relay record kind that authenticates the bundle by its own Personae
  signature; or the receive-only device gains a lane writer key.
- **F2. Roster gate on lane intake.** The lane admits any attested author; Mere's graph log
  admits only `SyncRoster` roots. Options: Mere supplies an admission filter to `join`; or the
  session's control-author check and DCGKA membership suffice.
- **F3. Pre-move graphs.** Under D3: refuse the old graph and re-key it fresh from its devices'
  current roster; or discard its sealed content.

## Progress

- 2026-11-06: findings read and spot-checked; D1 to D3; no source moved.
