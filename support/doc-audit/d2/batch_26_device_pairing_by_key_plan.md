# Batch 26 — device pairing by key plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md | current | yes | 18 | 18 | 0 | 0 |
| **Totals** |  |  | **18** | **18** | **0** | **0** |

**Totals: 1 doc, 18 claims checked (18 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `792967f8` (2026-10-01), plus the four machines read over
SSH on 2026-10-02. `archive_docs/` is excluded.

This batch exists because the plan is new. Its rulings (1 to 43) are recorded
in its §3, and its three corrections are carried as dated notes into the SSH
CA projection plan and the reachability rungs plan (R1, R2); they are not
counted again here.

## mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md

- disposition: current
- status line: "Status (2026-10-03): in progress. Assessed and ruled by Mark from 2026-10-01 to 2026-10-03 (rulings 1 to 43 below). D1 landed (`4963b489`); D1b's mere fix (M1) landed (`177b927c`), its fork fix (F1) is released as `mere-p2panda-net-0.7.5` (`1bec457e`, pushed), with knot and mere repinned locally and their pushes waiting on why iroh 1.3.0 holds a dead path longer (ruling 40); `connected` follows the gossip overlay (ruling 31) and is being finished, and the overlay's gap after restarts goes to its own lane (ruling 36); then D2." — accurate: yes
- claims checked: 18 — holds: 18, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The plan corrects the SSH CA plan's T2 and design claims and the
  reachability plan's R1 location and R2 announce claim; each correction is
  dated in that document and points here.

### Recommended action

- none for this record. The relay host (D6) and the macOS signed-app step
  (D2) are named as open in the plan.

### Notes

The claims checked: `DeviceRoster` and its fields
(`crates/dramatis/personae/src/carry/mod.rs:350`); `PairedDevice` in djinn
with `last_endpoint` and `relay_urls` (`ports/djinn/src/settings.rs:462,
468, 511`); `mint_host_cert` with only test callers (`ssh_ca.rs:276, 646`,
`tests/ssh_ca_live.rs:146`); `enroll-host`'s printed "host-key prompts still
apply" (`bin/personae-vault/certs.rs:185`); the CA derived from the profile
master (`certs.rs:34`) and a fresh master generated at random
(`bootstrap.rs:135`); `personae-vault ca` opening the vault through the
unlock ladder and `open_storage` creating the directory but `load` never
creating a profile (`bin/personae-vault/main.rs:118, 139`; `bootstrap.rs`);
the empty announce app data (`announce.rs:38-51`); and the four machines'
agents and certificate CAs (`ssh-add -L` on each, 2026-10-02), the
ThinkPad's avahi and firewalld state, and its and Q-PC's moved leases with
matching host keys (2026-10-01).

Added 2026-10-02 with rulings 36 to 39, three claims of the restart finding,
read in the source: `send_neighbor` sending only when its pending insert
succeeds (iroh-gossip 0.101.0 `proto/hyparview.rs:745-753`); a `Join` for a
topic with no state dropped (`proto/state.rs:247-275`); and `RETRY_RATE` at
5 s (`p2panda-net/src/sync/actors/topic_manager.rs:35`, the same at the
pinned `0a54ab82`).

Added 2026-10-03 with rulings 40 to 43, one claim of the release finding,
read in the source: Isometry's lenient decode of a peer operation body
(`crates/isonetry/src/campaign_space/space.rs:77`, at Isometry `cb9f89b`). The
release's pushed refs were read with `git ls-remote`.
