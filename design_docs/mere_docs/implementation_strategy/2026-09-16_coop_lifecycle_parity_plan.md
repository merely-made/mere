# Coop Lifecycle Parity Plan

**Date:** 2026-09-16
**Status:** in progress. Slice 1 of census lane I3. F1 to F4 landed as mere
`5e3850f7`; T1 to T3 landed as Turnstone `a1c9884` as an intermediate step.
The reading work below is open, so slice 1 is not done.
**Lane owner:** [suite census, I3 coop ceremony](../../2026-08-22_turnstone_suite_composition_and_capability_census.md)

## Purpose

I3 extracts a common invite/join/leave/reconnect contract only after two
concrete consumers demonstrate the same lifecycle facts. This slice brings
both consumers to that sequence and records, fact by fact, where they agree
and where they differ. It extracts nothing.

The two consumers:

| Consumer | Where it lives | Boundary |
|---|---|---|
| Turnstone place | `turnstone/src/place/`, T5a to T5c of `turnstone/design_docs/2026-07-28_turnstone_place_port_plan.md` | Product code; p2panda place transport; system clock |
| Practice fixture | `crates/moot/commons/examples/commons_practice_peer.rs`, `ports/graphshell/web/co_op_gateway.py`, `co_op.html`, `co_op_receipt.py` | Proof-only; fixed identities; fixed clock; loopback HTTP between two stores; receipt in the [projection grammar adoption plan](2026-08-15_projection_grammar_adoption_plan.md#shared-practice-proof-2026-09-04-verified-in-current-working-tree) |

The practice fixture is the census's "Woodshed peer comparison proof".
Woodshed contributes only the comparison export; no Woodshed code changes.

## Decisions (Mark, 2026-09-16)

- Parity first, extraction second. Slice 2 is planned after the facts table
  below is filled from receipts.
- Revocation is a product action in both consumers, not a test-only fact.
- The fixture reaches expiry through a proof-only advance-clock command; its
  clock stays deterministic.
- Turnstone invitations gain an optional grant lifetime, so both consumers can
  show a grant expiring while joined (T3).
- Revocation currently withdraws every operation a revoked writer authored,
  earlier ones included, because Gemot judges a grant at evaluation time
  rather than at each operation's authoring time. Judging each operation
  against the grant valid when it was written is planned as its own mere lane
  after slice 1.
- Slice 1 also makes revocation cut off reading: rekey the place's group on
  revoke and encrypt the shared graph lane. The tested revoke and lifetime work
  lands first as an intermediate step, with the revoked member's status saying
  reading is not yet revoked.
- Group-key frames travel on a dedicated group-key lane, keeping the recorded
  rule that Gemot decides who belongs and the group session decides who can
  read.
- Live members follow rotations through a refreshable key handle in Commons,
  shared by chat and the encrypted graph profile, rather than rejoining lanes
  after each rotation. Chat's keyring is otherwise fixed at construction and
  captured when its lane joins.
- The host owns epoch pruning for every lane sharing that handle: a lane
  reports the epochs its retained records still need, the host forgets only
  epochs no lane needs, prunes the group session itself, and then refreshes
  the handle. No lane mutates the shared handle to prune.
- Root runs the pin cascade for these mere changes rather than waiting for
  the lattice sync pass, which had not started.

## Assessment, 2026-09-16

I3's acceptance sequence against what each consumer demonstrates today:

| I3 fact | Turnstone place | Practice fixture |
|---|---|---|
| Invite a particular peer with bounded authority | Yes: pre-key-bound invitation, writer or reader | Yes: signed invitation and delegation |
| Refuse a wrong target or unauthorized grant | Yes: uninvited root refused at invite, reader writes and dials refused | Yes: wrong-space invitation, tampered delegation, pre-admission contribution refused |
| Join | Yes | Yes |
| Leave, stopping participation and keeping history | Yes: `Leave place` | No: only disconnect, which closes the store |
| Reconnect after restart, rechecking current admission and authority | Yes: `Reconnect place` rechecks retained membership | No: reopen and connect restore without a recheck |
| Duplicate replay adds no activity state | Yes, by sync; not separately receipted | Yes: duplicate-free reconciliation receipted |
| Expiry visible | Partly: an expired invitation is refused at admission; founding issues grants without expiry | No: clock fixed at 50 ms, grant expiry at 1000 ms never reached |
| Revocation visible | Partly: render-free projection withdrawal test; no product revoke action | No |

## Work

### Fixture

**F1. Leave.** A `leave` command removes the installed authority envelope and
keeps the store and every retained operation. Status reports `left` with the
retained counts. Contribution and sync after leave are refused as not joined.
Joining again requires installing an invitation again. Gateway action and
button `Leave space`, distinct from `Disconnect`.

**F2. Reconnect recheck.** `connect` and `reopen` evaluate the installed
authority at the fixture clock and against retained revocations before any
sync. An expired or revoked grant is refused with a named reason, no traffic is
sent, and status reports the verdict.

**F3. Advance clock.** A proof-only `advance_clock { now_ms }` command moves a
store-local clock, defaulting to today's 50 ms and never moving backwards.
Past the grant's expiry, status reports `expired`, and contribution and
reconnect are refused as expired. Gateway action and button marked proof-only.

**F4. Revoke.** The founder's `revoke_member` authors a revocation of the
member's delegation using Gemot's existing revocation semantics, retained and
carried over the same wire as operations. After sync, both peers retain the
member's operations and all of them read as revoked authority, earlier ones
included (see Decisions). The member's next reconnect is refused as revoked.
Gateway action and button `Revoke member`. Landed: revocation is recorded on a
real Gemot delegation lane in the fixture's store and carried over the existing
wire; a refused step sends no traffic.

### Turnstone

**T1. Revoke place member.** A founder-only palette action per member, shown
as a situational row naming the member's short root and access. The worker
authors a membership `Remove` and a revocation of the member's Commons and
projection delegations, stores both, and publishes both on the live lanes.
Landed: Gemot's delegation store admits only Moot-scoped statements, so the
graphshell-scoped projection grant is revoked in the holder's revocation
ledger, rebuilt at every bind from the Gemot fold. The revoked member's status shows both write
permissions not effective and a line naming the revocation; its writes and
projection dials are refused with a named reason; its `Reconnect place` is
refused by the existing membership recheck. The founder's projection withdraws
all of the member's operations without deleting them. Host-only in the ring.

**T2. Expiry reported.** An expired invitation's refusal already exists at
admission. Make it observable as a status line and event naming the expiry
time, and test it.

**T3. Optional grant lifetime.** `Invite to place` accepts an optional
trailing ` for <n>s|m|h|d` after the pre-key path; without it grants never
expire, so existing prompts and scenarios are unchanged. The lifetime bounds
the writer's Commons and projection delegations; a reader invitation carries
no grant, so a lifetime on it is refused with a named reason rather than
ignored. Past expiry the member's status names the expired grant and its time,
both write permissions read not effective, writes and projection dials are
refused as expired, and `Reconnect place` still reconnects, because membership
is intact and reading needs no delegation. That last point is a recorded
difference from the fixture, whose grant is its only authority.

### Reading

Assessed 2026-09-16. Chat already encrypts to the place's group through a
`DataKeyring` loaded from stickleback's `GroupSession`, which can add, remove
and rotate. Each change yields a control frame every member must process after
the domain has authenticated and causally ordered it, but no lane carries those
frames: only an invitation does, and only to the invitee. The shared graph
replica has no encryption. The Gemot lanes stay plaintext in this slice.

**R0. Add-invite suspicion: refuted, 2026-09-16.** Code reading suggested
that inviting a third member rotates the group epoch without already-joined
members receiving the frame. Turnstone `3262006`'s
`a_joined_member_reads_chat_authored_after_a_later_invite` shows otherwise:
stickleback's `GroupSession::add` hands the invitee the existing keys and never
rotates, the epoch was identical after founding and after both invites, and the
earlier member read chat authored after the later invite, live, with a positive
control and a seal/open probe across the two sessions. Three facts carry into
R2. `remove` and `update` do rotate, so every remaining member must receive
those frames. The founder's running worker loads its group and chat keyring
once at open and invites change only the copy on disk, so after a rotation the
running worker must refresh or it keeps encrypting to the old epoch. Chat
admission silently drops records it cannot decrypt and lane counters count only
accepted ones, so proving a member cannot read needs a positive control beside
it.

**R1. Group-key lane (mere).** A signed, causally ordered lane beside
stickleback's group session carrying `GroupControlFrame` and addressed
`GroupDirectFrame` records for add, remove and rotate, authenticated by the
authoring member's Personae root and applied through `GroupSession::process`
in order. Frames are size-bounded; duplicate and stale frames are ignored. It
joins like the other lanes and has a publish path. Landed as mere `71a767b8`:
`stickleback::GroupKeyLane`, whose drain takes a persist closure and writes
settled marks only after the host saved the session, and parks records that
arrive ahead of their author's previous record. Its tests show a remaining
member opening post-rotation ciphertext while the removed member gets an
unknown-epoch refusal. Late joiners know only their adder's sequence, which
holds while only the founder authors frames.

**R2. Rekey on invite and revoke (Turnstone).** The group-key lane becomes the
tenth place lane. Invite publishes its add dispatch on it; the invitation
still carries the welcome for the invitee. Revoke removes the member from the
group, rotates, and publishes both. Every member processes frames in order,
refreshes its chat keyring, and is nudged like any lane arrival. The reading
caveat line is removed once this works.

**E1. Encrypted graph profile (mere Commons).** Graph operation payloads are
sealed to the current group epoch with the same keyring approach chat uses,
and the projection decrypts with retained epochs, so a later member welcomed
with the retained epoch bundle reads history. Chat and this profile read keys
through one refreshable handle the host updates after draining the group-key
lane, so a joined lane admits new-epoch records and authors to the current
epoch without rejoining. The plaintext profile stays unchanged for the
practice fixture and other consumers. Landed as mere `af674f30`: `GroupKeys`
and `encrypted::EncryptedReplica` on lane `commons/graph/encrypted/v1`. Not
yet covered: refresh over a live LogSync lane, and a later member reading
encrypted graph history through the retained epoch bundle; both belong to
R2 and E2's tests.

**E1b. Host-owned pruning (mere Commons).** E1 found that chat's pruning
replaces the shared handle, so pruning chat would drop epochs the encrypted
graph still needs, and refreshing from the group session would restore
pruned epochs. Chat's pruning stops mutating the handle and reports what it
could forget; the encrypted graph reports the epochs its retained records
need; a Commons helper intersects the lanes' needs; the host prunes the group
session and refreshes the handle. Turnstone does not prune today, so its
wiring is not part of this slice.

**E2. Encrypted place graph (Turnstone).** New places open the shared graph
with the encrypted profile. Existing places live only in scratch test
directories and are not migrated.

**Cascade.** R1, E1 and E1b change mere, so knot-editor, mere's own knot-editor pin,
Woodshed's Redshank port and Turnstone move together, or ride the Paredros
session's wing-wide bump if one is running.

## Done conditions

- Fixture: `co_op_receipt.py` extended to cover leave and rejoin, expiry by
  advance-clock refusing contribution and reconnect, and revocation refusing
  the member's reconnect with its later operation retained but not effective.
  Native checks for leave, expiry and revocation added beside the existing six,
  which still pass. The process receipt JSON is recorded; the browser receipt
  for the new controls is a follow-on, not part of this slice.
- Turnstone: render-free tests that a revoked member sees the revocation live
  on a connected session, has writes and dials refused, and is refused on
  reconnect after restart, and that the founder's projection withdraws its
  later operation; a render-free test that a writer invited with a short
  lifetime sees its grant expire while connected, with writes and dials refused
  and reconnect still admitted; app tests for the revoke rows, the lifetime
  suffix and its refusal on a reader invitation, and the expiry report; the
  place tests and the four-window proof still pass.
- Reading: R0's result is recorded. A member invited earlier decrypts chat
  authored after a later invite. After revoke and rotation, the revoked member
  cannot decrypt chat or read shared graph nodes authored after the rotation,
  while remaining members can, proven render-free with a positive control. The
  four-window run sends chat after the reader joins and shows the joiner reads
  it. The reading caveat line is gone.
- The facts table below is filled from those receipts, with each consumer's
  exact status wording per fact, and every difference stated as a difference.
- `git diff --check` clean in both repositories.

## Facts table (to be filled from receipts)

| I3 fact | Turnstone evidence and wording | Fixture evidence and wording | Agree? |
|---|---|---|---|
| Invite with bounded authority | | | |
| Refuse wrong target or grant | | | |
| Join | | | |
| Leave | | | |
| Reconnect rechecks | | | |
| Duplicate replay | | | |
| Expiry visible | | | |
| Revocation visible | | | |

## Stop rules

- No shared contract type, crate or view is extracted in this slice.
- No change to Gemot semantics; both consumers use existing membership,
  delegation and revocation operations. Commons gains an encrypted graph
  profile beside the unchanged plaintext one and a refreshable key handle used
  by chat and that profile; stickleback gains the group-key lane; nothing else
  in either changes.
- The Gemot lanes stay plaintext: a removed member can still read membership
  and delegation facts, and the facts table says so.
- Authoring-time judgement of revoked writers' earlier operations is its own
  lane after this slice.
- Murm is not involved: it carries conversation exchange, not activity state.
- The fixture stays proof-only; fixed identities are not promoted to onboarding.
- Woodshed's repository is untouched.
