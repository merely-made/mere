# Coop Lifecycle Parity Plan

**Date:** 2026-09-16
**Status:** planned. Slice 1 of census lane I3.
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
member's operations; any authored after revocation read as revoked authority
and leave the effective view. The member's next reconnect is refused as
revoked. Gateway action and button `Revoke member`.

### Turnstone

**T1. Revoke place member.** A founder-only palette action per member, shown
as a situational row naming the member's short root and access. The worker
authors a membership `Remove` and a revocation of the member's Commons and
projection delegations through `author_revoke`, stores both, and publishes
both on the live lanes. The revoked member's status shows both write
permissions not effective and a line naming the revocation; its writes and
projection dials are refused with a named reason; its `Reconnect place` is
refused by the existing membership recheck. The founder's projection withdraws
the member's later operations without deleting them. Host-only in the ring.

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
- No change to Commons or Gemot semantics; both consumers use existing
  membership, delegation and revocation operations.
- Murm is not involved: it carries conversation exchange, not activity state.
- The fixture stays proof-only; fixed identities are not promoted to onboarding.
- Woodshed's repository is untouched.
