# Batch 30 — upstream candidates ledger (new research ledger)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/research/2026-10-03_upstream_candidates_ledger.md | current | yes | 12 | 11 | 0 | 1 |
| **Totals** |  |  | **12** | **11** | **0** | **1** |

**Totals: 1 doc, 12 claims checked (11 holds, 0 stale, 1 unverifiable), 0 contradictions.**

Audit base: Mere `fdb02bd3` (2026-10-03), with this pass's edits in the
working tree: the new ledger, its `DOC_README.md` entry at the head of the
`mere_docs/research/` list, and the pairing plan's rulings 44 to 47. Sources
read: iroh 1.2.0 and 1.3.0 and iroh-gossip 0.101.0 from the local Cargo
registry; `crates/p2panda` at `0a54ab82`; the Burn 0.22 migration plan. No
web sources; `archive_docs/` is excluded.

This batch exists because the document is new.

## mere_docs/research/2026-10-03_upstream_candidates_ledger.md

- disposition: current
- status line: "Status (2026-10-03): open; eight items, none raised. Kept by ruling 46 of the device pairing plan: noted for a later review, raised only after a release passes them by." — accurate: yes
- claims checked: 12 — holds: 11, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The claims checked: `ACTOR_MAX_IDLE_TIMEOUT` at 60 s (iroh 1.3.0
`socket/remote_map/remote_state.rs:74`, 1.2.0 `:73`) and its reset when the
actor is not idle (`:265-269`); `handle_connection_close` (`:475-491`)
removing the connection without changing path status; the abandon handler
byte-identical at 1.3.0 `:600-618` and 1.2.0 `:595-613`, with the comment
"once no connections have any path" (`:610`); iroh-gossip's `send_neighbor`
gated on its pending insert (`proto/hyparview.rs:745-753`), a `Join` for a
topic with no state dropped (`proto/state.rs:247-275`), and `accept_conn`
swapping the active connection (`net.rs:772`); p2panda's `set_topics` as the
place topic watchers update (`p2panda-net/src/address_book/actor.rs:131-145`
at `0a54ab82`); the CubeCL `persistence` chain and "no consumer-side switch"
(Burn plan §13.3), the retirement reading (§13.13) and production on
`0.22.0-pre.2` (its status annotation). The measured figures in items 1, 3
and 5 are the pairing plan's §6 records, not rechecked here. Unverifiable:
item 8's "last active 2026-07-28" for upstream PR #7, carried from the
pairing plan's §6 and not checked against GitHub in this pass.
