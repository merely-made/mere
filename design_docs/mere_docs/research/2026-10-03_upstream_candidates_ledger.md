# Upstream Candidates Ledger

**Date**: 2026-10-03
**Status (2026-10-08)**: open; eleven items, none raised. Kept by ruling 46 of
the device pairing plan: noted for a later review, raised only after a
release passes them by. Item 9 (argon2) comes from the vault lock plan's
ruling 33.
**Scope**: defects and rough edges found in the stack's fastest-moving
dependencies (iroh and iroh-gossip, p2panda, and Burn's CubeCL) that the
upstream projects may want to hear about, with what we carry meanwhile.
Item 9 extends it to a security dependency, argon2 (vault lock ruling 33).

**Related**:

- [device pairing by key plan](../implementation_strategy/2026-10-02_device_pairing_by_key_plan.md):
  ruling 46, and the measurements behind items 1 to 6 (its §6).
- [Burn 0.22 migration plan](../implementation_strategy/2026-08-09_burn_0_22_migration_plan.md):
  item 7 (its §13.3).
- [vault lock plan](../../dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md):
  item 9 (its ruling 33).

---

## 1. The rule (ruling 46)

Asked whether iroh should hear about the `remote_info` finding, Mark said:
**"Hmm. Burn, p2panda, and iroh are key dependencies developing hard. They
may appreciate a heads up in the form of an issue. But they likely get a
lot, and I would hate to be part of overwhelming them. That’s a big problem
for open source developers today: tons and tons of good faith issues and prs
that strike like a cholesterol glut in a project’s heart. So that’s part of
my reluctance. Plus every issue is so dry and technical… i would just not
write them like that unless I knew they were using LLMs to analyze the issue
anyway (like prns). Keep note of these issues for a review later. If a
(pre)release of any of the three goes by without the issue being addressed,
it is then worth bringing up in a chill way that does not demand full mental
bandwidth from stressed developers (i.e. describing the issue and why it
matters in plain english, with a few key technical points as necessary)."**

So, for each item:

- It stays here until a release or prerelease of its project ships without
  addressing it. Only then does it go to Mark, as a short plain-English note:
  what happens, why it matters to someone using the library, and a few
  technical points. Nothing is posted without Mark reading it and saying so.
- Each item records the release it was last checked against. When a release
  of iroh, iroh-gossip, p2panda or CubeCL lands in a repin, its items are
  checked again and the row updated.
- An item that turns out to be by design, or already reported, is closed
  with that finding, not deleted.

## 2. Items

### 1. iroh: a closed connection's paths stay "Active"

- **What happens:** after the last connection to a peer closes, `remote_info`
  keeps reporting that peer's paths as `Active` ("in active use") until the
  peer's remote actor idles out, 60 s after its last queued message
  (`ACTOR_MAX_IDLE_TIMEOUT`, `socket/remote_map/remote_state.rs:74`). Every
  `remote_info` call queues a message and so restarts that minute; under
  load the actor runs late and the gap stretches. Measured: a killed peer
  read active for 71 to 90 s on a quiet machine and 100 to 312 s under build
  load, the same on 1.2.0 and 1.3.0.
- **Why it matters:** "is this peer reachable now" is the obvious use of
  `remote_info`, and polling it keeps the stale answer alive.
- **Likely fix (reading):** mark a connection's paths inactive in
  `handle_connection_close` (`remote_state.rs:475-491`), which removes the
  connection without touching path status.
- **What we carry:** pairing ruling 31 (gossip decides for overlay peers);
  ruling 47's connection-event liveness for the rest.
- **Last checked:** iroh 1.3.0.

### 2. iroh: the abandon check reads only the closing connection

- **What happens:** `NoqPathEvent::Abandoned` marks an address abandoned when
  the *closing* connection's paths no longer reach it, although its comment
  says "once no connections have any path". During simultaneous dials every
  address of a peer that is still delivering can read inactive for 45 s or
  more (`remote_state.rs:600-618` in 1.3.0, byte-identical at `:595-613` in
  1.2.0).
- **Why it matters:** two peers dialling each other at once is ordinary; the
  path state then says the link is dead while it carries traffic.
- **What we carry:** pairing rulings 30 and 31.
- **Last checked:** iroh 1.3.0.

### 3. iroh-gossip: a stale pending neighbour request after a restart

- **What happens:** when a peer restarts, the side that stayed up can hold a
  pending `Neighbor` request for it from an earlier exchange, which nothing
  clears while the connection lives. `send_neighbor` sends only when its
  pending insert succeeds (`proto/hyparview.rs:745-753`), so the restarted
  peer's `Join` gets no answer and it never gains a neighbour; the survivor,
  which swaps the connection silently (`net.rs:772-801`), never sees it go.
  Measured in 12 of 12 runs: it follows whichever restart comes second, and
  data from the restarted side then arrives 17 to 113 s late, through sync
  retries.
- **Why it matters:** a device that crashes and restarts can sit half joined
  to the overlay for minutes.
- **What we carry:** pairing ruling 36's overlay-fix lane. *2026-10-04:*
  the ruled fix clears a peer's pending entry in `on_join` when it is
  already an active neighbour (pairing rulings 64, 65), but no fork is made
  while we wait for upstream (ruling 72). A `Neighbor` overtaking a `Join`
  forms the entry too, not only a one-sided join.
- **Upstream already has:** issue #172 (open, 2026-09-29) reports the same
  last step reached by another route, and PR #159 (open, outside
  contributor, conflicting, unreviewed) carries our exact `on_join` change
  inside a 2,435-line rework. A note from us would add our route (a
  killed process on one topic, message order) to #172 or #159.
- **Last checked:** iroh-gossip 0.101.0 (upstream `v0.101.0`, `2ce78afe`);
  upstream `main` `2885dd9f`, unchanged in `hyparview.rs` (2026-10-04).

### 4. iroh-gossip: a `Join` for an unsubscribed topic is dropped

- **What happens:** a `Join` arriving for a topic the receiver has not yet
  subscribed to is dropped without a reply, and nothing retries it
  (`proto/state.rs:247-275`).
- **Why it matters:** only when one side never joins back; in two-way setups
  the later subscriber's own join heals it. Possibly by design; a reading.
- **What we carry:** pairing ruling 38 (the receiver subscribes first in
  one-way tests).
- **Last checked:** iroh-gossip 0.101.0.

### 5. iroh: `close()` can stall past 10 s after simultaneous dials

- **What happens:** the endpoint's `close()` returned in 2 to 15 ms after an
  ordinary link, but stalled past 10 s three times in one suite run, and in
  2 of 10 and 1 of 10 later runs, after both sides had dialled at once. The
  cause is not determined.
- **Why it matters:** shutdown paths that wait on `close()` hang.
- **What we carry:** the transport's simultaneous-dial test logs a stall and
  moves on.
- **Last checked:** iroh 1.2.0 (seen); not yet looked for on 1.3.0.

### 6. p2panda: topic watchers miss a node's new record

- **What happens:** the address book recomputes topic watchers only when
  topics are written (`p2panda-net/src/address_book/actor.rs:131-145`, at
  `0a54ab82`), not when a node's record or transport info is, which is where
  mDNS writes. A paired node tagged before any record existed is never
  joined: first contact by mDNS alone fails.
- **Why it matters:** any app that pairs devices first and lets local
  discovery find them later.
- **What we carry:** F1 in the `mark-ik/p2panda` fork, released as
  `mere-p2panda-net-0.7.5` (pairing rulings 19 to 21; ruling 21 kept it out
  of upstream "for now"), and M1 in mere-transport.
- **Last checked:** p2panda upstream `main` as merged into 0.7.5
  (`e11d590d`).

### 7. CubeCL: `persistence` cannot be turned off without a patch

- **What happens:** `persistence` is on by default through `cubecl-wgpu`'s
  defaults (`cubecl-runtime/persistence` → `cubecl-environment/persistence`
  → `turso`), and no consumer-side switch exists; the Burn plan found four
  ways to turn it off, every one a patch (its §13.3).
- **Why it matters:** it brings a database engine and its transitive crates
  into every graph that uses the wgpu backend.
- **What we carry:** nothing yet; production is on `0.22.0-pre.2`. The pre.4
  migration will carry a manifest-only runtime patch, retired "when upstream
  makes persistence optional without the forcing edge" (§13.13, a reading
  there).
- **Last checked:** Burn 0.22 pre.4 and its CubeCL, per that plan.

### 8. iroh-mdns-address-lookup: per-interface multicast (watch only)

- **What happens:** upstream PR #7 (per-interface multicast sockets) is
  unmerged, last active 2026-07-28; on this laptop both it and our port of
  it fail upstream's `mdns_subscribe` test while stock passes.
- **Why it matters to us:** multi-homed machines (the WSL adapter) can bind
  the wrong interface. This is someone else's open PR, so there is nothing
  of ours to raise; the item records our Windows failure for when the PR
  moves (pairing ruling 26 holds the diagnosis).
- **Last checked:** `iroh-mdns-address-lookup` 0.6.0.

### 9. argon2: working memory freed uncleared

- **What happens:** `Argon2::hash_password_into` allocates the algorithm's
  memory blocks (about 19 MiB at our parameters) and frees them without
  clearing. In 0.5.3 the `Vec<Block>` drops uncleared even with the
  `zeroize` feature (`src/lib.rs:229-232`); in 0.6.0-rc.8 `Blocks`'s `Drop`
  deallocates without zeroizing (`src/block.rs:190-200`). The final blocks
  suffice to recompute the derived key. Read in source, not measured.
- **Why it matters to us:** the vault lock's passphrase unlock derives its
  key through argon2, and a lock should leave no key material behind.
- **What we carry:** vault lock ruling 33. We call argon2's own public
  `hash_password_into_with_memory` with a buffer we zeroize, with argon2's
  `zeroize` feature on. That is a public API, not a patch.
- **Last checked:** argon2 0.5.3 (in the lock) and 0.6.0-rc.8.

### 10. iroh: a bind's builder, secret key included, stays in its future

- **What happens:** `Endpoint::builder(..).secret_key(..).bind()` takes the
  builder by value, so the `SecretKey`'s bytes sit in the bind future's
  state and the caller's task frame. When those are freed the bytes are
  not cleared: 7720 bytes (the tokio task cell, key at offset 808) and
  7736 (tokio's debug-build box of the large future, offset 5736). The
  same holds further in:
  - `iroh::socket::bind` (3464 bytes, offsets 2792 and 3112);
  - `RelayTransport::new` in `Transports::bind` (4520, offset 704);
  - `endpoint::Builder::address_lookup` (240, offset 208);
  - `portmapper::Client::new` (3400, intermittent, freed on a nat_pmp
    error);
  - iroh-gossip's `net::Builder::spawn` (5512, offset 4072).

  `SecretKey` clears itself on drop; only moved copies remain. Measured
  with a test-only tracking allocator.
- **Why it matters to us:** the vault lock (rulings 49 to 51) wants no
  key material left after a lock. These copies are freed, not live, but
  their contents survive until reused.
- **What we carry:** nothing; we measure against an iroh-only baseline
  (`crates/murm/transport/tests/seed_residue.rs`).
- **Last checked:** iroh 1.3.0, iroh-gossip 0.101.0, portmapper 0.19.3.

### 11. p2panda-net: the signing key moves through actor futures

- **What happens:** p2panda-net's by-value `signing_key(SigningKey)` API
  leaves the key's bytes in futures and actor state that are freed
  uncleared:
  - `iroh_endpoint::Builder::spawn` (2360 bytes, offset 200);
  - `gossip::Builder::spawn` (3680, an `Endpoint` clone's args, offset
    680);
  - ractor start and run futures for the `IrohEndpoint` actor (448, 544,
    736) and for `GossipManager` (648, 744, 2584).

  An actor call that boxes the 392-byte `ToIrohEndpoint` enum copies
  uninitialized bytes from a caller stack that still held key bytes from
  bind (offsets 120 and 280). A 1824-byte block goes with it, untraced.
  Measured.
- **Why it matters to us:** as item 10.
- **What we carry:** mere's transport keeps iroh's endpoint handle from
  bind and asks the actor nothing afterwards (vault lock rulings 52 and
  54), so the 392 and 1824 blocks no longer appear in transport runs. The
  other blocks are p2panda-net's own and remain.
- **With mDNS on (2026-10-08, vault lock ruling 60).**
  - Spawning `MdnsDiscovery` (Active, after the endpoint and before gossip,
    as Mere's default host policy does) adds 4 more blocks freed
    uncleared: 472, 568, 784 and 2424. They appear in every run of
    p2panda-net alone.
  - Measured as not iroh's: iroh's `MdnsAddressLookup` 0.6.0, built the
    same way from the public id with no p2panda actor, adds nothing beyond
    iroh alone. The lookup is never handed the key.
  - So the copies come from p2panda-net's `MdnsActor` layer, presumably the
    same ractor boxing of a key-bearing stack as above. That is inferred,
    not traced.
  - mere-transport's `seed_residue.rs` now runs this shape.
- **Last checked:** mere-p2panda-net 0.7.5 and p2panda-core 0.7.1 (fork
  tag `mere-p2panda-net-0.7.5`, `1bec457`), ractor 0.16.5.

## 3. Progress

**2026-10-03.** Opened with items 1 to 8 from the pairing plan and the Burn
plan. None raised.

**2026-10-05.** Item 9 (argon2) added from the vault lock plan's ruling 33.
Not raised.

**2026-10-06.** Items 10 (iroh) and 11 (p2panda-net) added from the vault lock
plan's rulings 49 to 52. Not raised.

**2026-10-08.** Item 11 extended with p2panda-net's mDNS blocks (vault lock
ruling 60), measured apart from iroh's mDNS lookup. Not raised.
