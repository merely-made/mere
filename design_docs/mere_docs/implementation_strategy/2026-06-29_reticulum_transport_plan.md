# Reticulum transport plan

**Status (2026-10-06):** P0/P1 and the bounded test row landed on the
Beechat `reticulum` 0.1.0 crate on 2026-07-01. `be99eadb` (2026-07-15,
retinue's R5) replaced that backend with retinue behind the same trait
(`reticulum = ["dep:retinue", "dep:hkdf", "dep:sha2"]` in
`crates/murm/transport/Cargo.toml`, retinue 0.2.0 by git rev), and
`a69e4867` (2026-07-27) revised the identity and the binding: the Ed25519 half
is the master seed itself, and announce app data is empty, the 96-byte binding
read only as legacy. The Direction section's trigger has fired, but Phase 3's
decision is unrecorded; the feature is default-off, enabled by `commons-spine`
only as a dev-dependency and by the `murm-direct-phy` probe as a normal one.
Open: a forged legacy-binding test, a README section on ALPN mapping, announce
discovery and binding, the DOC_README update, and the Phase 3 decision itself.

Plan for adding an optional `ReticulumTransport` backend to the Mere `transport`
crate (`crates/murm/transport`).

## Goal

Implement a `Transport`-trait backend that runs Mere's bilateral peer-to-peer
lane over [Reticulum](https://reticulum.network/) packet links, using the Beechat
Rust port [`reticulum`](https://crates.io/crates/reticulum) v0.1.0. The probe is
limited to bilateral stream connectivity: one `connect(peer, alpn)` and one
`accept(alpn)` yielding an `AsyncRead + AsyncWrite` stream. Sync (gossip / RBSR
/ LogSync) and blob transfer remain iroh-only for now.

**Corrected 2026-10-06 (S14 pass):** the backend is no longer Beechat's
crate. `be99eadb` (2026-07-15) replaced it with retinue: the feature is
`reticulum = ["dep:retinue", "dep:hkdf", "dep:sha2"]`
(`crates/murm/transport/Cargo.toml`), and the workspace pins retinue 0.2.0 at
git rev `fa4f925`. Retinue's v0 plan records "R5 — Mere adoption. DONE
2026-07-15".

## What is in scope

- A new `ReticulumTransport` module implementing `Transport`.
- Deterministic Reticulum identity derived from the Mere master seed (same seed →
  same Reticulum destination across restarts).
- Authenticated binding between a Reticulum announce and a Mere `PeerID`.
- ALPN → Reticulum destination namespace mapping.
- Bidirectional byte stream over a Reticulum `Link`.
- In-process tests using two transports on a local TCP loopback.
- Optional feature-gating so the dependency is default-off.

## What is out of scope

- Gossip, live-sync, offline catch-up, or blob transfer over Reticulum.
- Serial / RNode / LoRa hardware interfaces for the first probe.
- Meshtastic / MeshCore bridges (a different integration shape).

## Direction (2026-07-06): own implementation when this lane gets investment

Decided with Mark: if the Reticulum lane graduates beyond this probe, Mere stewards
its **own Rust implementation** rather than adopting Beechat or FreeTAK as the
long-term dependency (the misfin posture, applied again). Grounding, from the
[LXMF research brief](../research/2026-07-06_lxmf_key_addressed_mail_research.md)
plus a stewardship check the same day:

- The Beechat `reticulum` crate (MIT) has sat at 0.1.0 since 2025-10; FreeTAK's
  `reticulum-rs` (EPL-2.0) is active but daemon/enterprise-shaped and days-old at
  0.6.x. Neither optimizes for a library embed behind Mere's `Transport` trait.
- Upstream itself is in flux: the reference implementation's license changed in
  April 2025 (MIT plus an anti-AI clause, the "Reticulum License"), Mark Qvist
  stepped back from RNS development in December 2025, and community forks exist
  (RetiNet, Reticulum_CE). The **protocol is public domain**, which makes a
  spec-based implementation the cleanest ownership path.
- Reference discipline: implement from the public-domain protocol spec + manual;
  read Beechat (MIT) freely; read FreeTAK (EPL-2.0) for technique only, never
  copied text; treat the Python reference as a black-box interop oracle
  (mixed-runtime smoke tests against `rnsd`) rather than a code reference, given
  its license posture.

  **Corrected 2026-10-06 (S14 pass):** retinue's current notice no longer
  matches the black-box-only rule. Its `THIRD_PARTY_NOTICES.md`, section
  "Reticulum", reports a first limited review of RNS implementation source on
  2026-09-26, says no RNS code was copied or translated, and calls comparative
  review the current scope, adaptation being a separate decision.

  **Open, raised by the S14 pass (2026-10-06):** should this reference
  discipline keep its own wording, now that retinue's notice states the
  current posture? Options: point this section at retinue's notice instead of
  restating it; keep it as Mere's record of the 2026-07-06 rule, with the
  notice cited beside it.
- Scope: **endpoint-first**, wire-compatible with RNS 1.3.x — identity, announce,
  link, resource, TCP interface first; transport-node routing and RNode/LoRa
  interfaces later; LXMF-wire optional on top. A Mere node needs to be a
  Reticulum endpoint, not a router, to interoperate.
- Trigger unchanged: the probe stays pinned to Beechat 0.1.0 (it works) until the
  lane gets real investment; the own-impl replaces it behind the same trait when
  that day comes.
- **Named (Mark, 2026-07-06): `retinue`** — the company that travels with a
  person, which is what a persona's transport is; echoes "reticulum" while
  staying a plain word. **Scaffolded + reserved same day**: repo at
  `repos/retinue` (dual MIT/Apache-2.0), `retinue` 0.0.1 published to
  crates.io as the name reservation, v0 plan at
  `repos/retinue/design_docs/2026-07-06_retinue_v0_plan.md` (phases R0
  oracle-harness/primitives → R5 Mere adoption). Standalone-sibling shape:
  own repo, crates.io-only dep, one-way (Mere consumes it).

**Corrected 2026-10-06 (S14 pass):** the trigger fired. Mere's own
implementation replaced Beechat behind the same trait in `be99eadb`
(2026-07-15), so the probe is no longer pinned to Beechat 0.1.0. Retinue's
README gives its licence as MPL-2.0, not dual MIT/Apache-2.0, and Mere takes
retinue by git rev in the root `Cargo.toml`, not as a crates.io-only
dependency.

## Findings

### From source reading (`reticulum` v0.1.0)

- `PrivateIdentity` is a dual-key identity: X25519 `StaticSecret` + Ed25519
  `SigningKey`. The public `Identity` contains both public keys, and the
  destination `address_hash` is computed over both. A Mere `PeerID`, which is a
  32-byte Ed25519 verifying key, does **not** contain the X25519 public key and
  therefore cannot be used to synthesize another peer's destination hash.
- Discovery is announce-based. A peer calls `send_announce(destination,
  app_data)` on its local destination; other peers receive the announce via
  `recv_announces()`. The announce carries the sender's public identity and an
  optional `app_data` blob.
- An outbound link is created with `transport.link(announce.destination.lock().await.desc)`,
  where the descriptor comes from a validated announce.
- Link data arrives as `LinkEvent::Data(payload)` from `out_link_events()`
  (outbound side) or `in_link_events()` (inbound side), and is sent with
  `link.data_packet(payload)` followed by `transport.send_packet(packet)`.
- The `buffer` module exposes only low-level primitives (`StaticBuffer`,
  `OutputBuffer`, `InputBuffer`). There is no `BufferReader`/`BufferWriter` stream
  abstraction in v0.1.0; the stream wrapper must be built directly on link data
  packets.
- `Transport::new(config)` requires a `PrivateIdentity`, a name, and a broadcast
  flag. Interfaces are attached through `iface_manager().lock().await.spawn(...)`.
- The crate uses `ed25519-dalek` 2.1.1 and `tokio` 1.x, matching Mere's existing
  stack. Its build script needs `protoc` for the Kaonic gRPC protobuf files.

**Corrected 2026-10-06 (S14 pass):** these findings describe Beechat's crate,
which `be99eadb` replaced with retinue. The `protoc` requirement was Beechat's
`build.rs`; the retinue crate has no `build.rs` (only its firmware has one)
and no tonic or prost dependency.

### Identity mapping

Mere's master key is a single Ed25519 keypair. Reticulum needs both X25519 and
Ed25519 keys. We derive both deterministically from the 32-byte Ed25519 seed
using HKDF-SHA256 with a Mere-specific context string, producing a reproducible
`PrivateIdentity` for each Mere seed. The Mere `PeerID` is still computed from
only the Ed25519 verifying key, so it is stable with respect to the other Mere
transports.

**Corrected 2026-10-06 (S14 pass):** only the X25519 half is HKDF-derived
now. The Ed25519 half is the Mere master seed itself (`derive_identity` in
`crates/murm/transport/src/reticulum_transport/keys.rs`), changed in
`a69e4867` (2026-07-27).

### Authenticated PeerID binding

Because a destination hash cannot be computed from a `PeerID`, `connect(peer,
alpn)` must learn the peer's destination from an announce. To prevent a peer
from announcing someone else's `PeerID`, every announce's `app_data` carries:

```text
app_data = PeerID || signature
signature  = ed25519_sign(master_signing_key,
                           reticulum_identity_public_keys || PeerID || ALPN)
```

The receiver:

1. Validates the Reticulum announce signature.
2. Parses `PeerID` from the first 32 bytes of `app_data`.
3. Verifies the Mere master signature using the recovered `PeerID` public key.
4. Stores `PeerID → (ALPN, DestinationDesc)` in a local address book.

Only after this binding succeeds can `connect(peer, alpn)` resolve the peer.

**Corrected 2026-10-06 (S14 pass):** announce app data is now intentionally
empty: the signed retinue identity already carries the Mere public key, and the
96-byte binding made a valid announce too large for a 255-byte LoRa frame. The
`PeerID || signature` form is read only as legacy
(`crates/murm/transport/src/reticulum_transport/announce.rs`), changed in
`a69e4867` (2026-07-27).

### ALPN mapping

Each ALPN string maps to a Reticulum destination name. For example:

- `mere/cable/v1` → `DestinationName::new("mere", "cable.v1")`
- `mere/coop/v1` → `DestinationName::new("mere", "coop.v1")`

Per ALPN, the transport registers an incoming destination and periodically
announces it.

### Stream mapping

`ReticulumStream` implements `AsyncRead + AsyncWrite` over a `Link`:

- Write: buffer bytes, chunk into Reticulum-sized payloads, build data packets
  with `link.data_packet(chunk)`, and send them with `transport.send_packet`.
- Read: a background task drains the appropriate link event receiver and pushes
  payloads into an `mpsc` queue; `poll_read` pulls from the queue.
- Flush: drain the write buffer and await hand-off to the transport.
- Shutdown: close the Reticulum link.

## Progress

- 2026-06-21 — Research brief on off-grid / LoRa transports completed; decided
  p2panda-core can ride Reticulum but p2panda-net cannot run on packet-radio
  interfaces.
- 2026-06-22 — Initial implementation plan drafted and approved.
- 2026-06-29 — Plan corrected after source review:
  - Identity mapping changed from "derive destination from `PeerID`" to
    "deterministically derive full dual-key Reticulum identity from master
    seed".
  - Discovery model changed from "synthesize destination" to "announce-based
    discovery with authenticated `PeerID` binding".
  - Stream implementation acknowledged to use link data packets directly; the
    `buffer` module does not provide a stream abstraction in v0.1.0.
- 2026-06-29 — Dependency and feature flag landed:
  - `reticulum = "0.1"` added to workspace root `[workspace.dependencies]`.
  - Optional `reticulum` feature added to `crates/murm/transport/Cargo.toml`.
  - `cargo check -p transport --features reticulum` passes with `PROTOC` set to
    a local `protoc.exe`.

  **Corrected 2026-10-06 (S14 pass):** `be99eadb` (2026-07-15) replaced the
  `reticulum = "0.1"` dependency with retinue; see the correction under the
  Goal.
- 2026-07-01 — **P0/P1 implemented and verified (this is the real green).** A
  review found the 2026-06-29 state was a *false green*: `reticulum_transport.rs`
  was committed but never declared in `lib.rs`, so `--features reticulum` compiled
  the deps and skipped the (non-compiling, half-written) module. The `Transport`
  impl and five functions `bind_inner` called (`derive_identity`,
  `destination_name_for_alpn`, `announce_listener`, `announce_sender`,
  `build_app_data`) did not exist, and `send_announce_now` could never run
  (`master_keypair()` always errored). Fixed and completed against the real
  `reticulum` v0.1.0 source (read from the crate cache):
  - **Wired in.** `#[cfg(feature = "reticulum")] pub mod reticulum_transport;` +
    re-exports in `lib.rs`. Split into a module dir
    (`reticulum_transport/{keys,announce,stream}.rs` + mod root) to stay under the
    600-LOC ceiling.
  - **Transport impl.** `connect` resolves a peer from the announce-populated
    address book, opens an out-link, waits for activation, and bridges it;
    `accept` waits for an inbound-link activation on the ALPN's destination and
    bridges it. `ReticulumStream` is a tokio `DuplexStream` driven by two relay
    tasks (aborted on drop).
  - **`send_announce_now` fixed** by precomputing each ALPN's signed `app_data`
    once at bind and storing it, so no master secret is retained after
    construction.
  - **API corrections vs the Findings above** (real v0.1.0): `Transport::new` is
    sync; `add_destination` takes `&mut self` (so destinations are registered
    *before* the stack goes behind `Arc`); the Reticulum identity is built via
    `PrivateIdentity::new_from_hex_string` from the HKDF-derived 64 bytes (avoids
    adding an `ed25519-dalek` direct dep); sending uses
    `transport.send_to_out_links` / `send_to_in_links`, receiving uses
    `out_link_events()` / `in_link_events()` (`LinkEventData { id, address_hash,
    event }`); **announces carry only the 10-byte name hash**, so ALPN matching
    keys on `as_name_hash_slice()`, not the full 32-byte `Hash`;
    `Ed25519Signature::from_bytes` is infallible (returns `Self`).
  - **protoc is genuinely required.** `reticulum`'s `build.rs` unconditionally runs
    `tonic-build` on the Kaonic gRPC proto (no feature gate), so `protoc` must be
    present to build the feature. It was not on PATH; the build used a pinned
    prebuilt `protoc` via the `PROTOC` env var. This is a real build/CI
    prerequisite, recorded in the risk table.

    **Corrected 2026-10-06 (S14 pass):** the requirement left with Beechat's
    crate. Retinue has no `build.rs` outside its firmware and no tonic or
    prost dependency.
  - **Verified.** `cargo check -p transport --features reticulum --tests` green
    (15s); `cargo clippy` clean; `cargo test` green — 3 tests, incl.
    `bilateral_round_trip_over_tcp_loopback` (two instances discover each other by
    authenticated announce, establish a link, and round-trip `hello`/`world`),
    finishing in 0.64s. Mere-side changes uncommitted (concurrent meerkat/orrery
    work in the tree).
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere 535bca11, from the D2 record in support/doc-audit/d2/batch_41_s14_phase_b3.md: the Beechat backend recorded as replaced by retinue (`be99eadb`) and the identity and binding as revised (`a69e4867`), the protoc, licence and crates.io claims corrected, and the reference discipline's relation to retinue's notice raised as an open question.

## Phases and done-conditions

### Phase 0 — API and dependency verification (complete, for real 2026-07-01)

Done when:
- `cargo check -p transport --features reticulum` passes **with the module wired
  into `lib.rs`** (the 2026-06-29 pass did not meet this; the module was orphaned).
- Exact `reticulum` APIs for identity, destination, announce, link, and packet
  events are documented in Findings (corrected 2026-07-01).
- Identity derivation strategy is chosen (HKDF-SHA256 → `new_from_hex_string`).

### Phase 1 — Core transport skeleton (complete, verified 2026-07-01)

Done when:
- `ReticulumTransport` struct owns `reticulum::Transport`, local `PeerID`,
  per-ALPN destinations, and an authenticated peer→destination map. ✓
- `ReticulumStream` implements `AsyncRead + AsyncWrite` over link data packets. ✓
- `Transport` trait is implemented for `ReticulumTransport`. ✓ (`connect` / `accept`)
- Deterministic dual-key identity derivation and ALPN→destination mapping are
  implemented. ✓

### Phase 2 — Tests (mostly complete 2026-07-01: 3 tests green)

Done when:
- TCP loopback round-trip test passes between two transports. ✓
  (`bilateral_round_trip_over_tcp_loopback`)
- Unregistered-ALPN and deterministic-identity/peer-id tests pass. ✓
  (`accept_unregistered_alpn_errors`, `derived_identity_is_stable_and_peer_id_matches`)
- `cargo test -p transport --features reticulum` is green. ✓
- Remaining: a standalone binding-authentication test (a forged/mismatched
  `app_data` must be rejected — currently only exercised implicitly, since
  `connect` succeeds only when the binding verifies).

**Corrected 2026-10-06 (S14 pass):** the module's `tests.rs` now holds five
tests, the three named above among them, and still no forged or mismatched
binding test. Since `a69e4867` the `app_data` binding is the legacy form (see
the correction under "Authenticated PeerID binding"), so the remaining test is
a forged legacy binding.

### Phase 3 — Documentation and decision gate

Done when:
- `crates/murm/transport/README.md` documents the optional backend, the ALPN
  mapping, announce discovery, authenticated binding, and sync limitations.
- This plan's Progress section is updated with implementation results.
- `design_docs/DOC_README.md` is updated.
- Decision recorded: keep feature-flagged, wire into host config, or pause on
  blockers.

**Corrected 2026-10-06 (S14 pass):** `crates/murm/transport/README.md` covers
only the feature row, not the ALPN mapping, announce discovery or binding. No
decision is recorded; in the tree the feature stays default-off, enabled by
`commons-spine` as a dev-dependency and by the `murm-direct-phy` probe as a
normal one.

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| `reticulum` v0.1.0 has minimal docs and is pre-1.0 | Source is the spec; keep feature optional and default-off. |
| `protoc` required at build time | Document requirement; set `PROTOC` in dev/CI. |
| No stream abstraction in the crate | Build directly on `LinkEvent::Data` / `link.data_packet()`. |
| `PeerID`-to-destination synthesis is impossible | Use authenticated announce cache instead of synthesis. |
| Identity derivation mismatch | Use deterministic HKDF from the Ed25519 seed; test reproducibility. |
| Lock-heavy `Arc<Mutex<_>>` API | Minimize critical sections; run event drains in background tasks. |

**Corrected 2026-10-06 (S14 pass):** the `reticulum` v0.1.0 and `protoc` rows
describe Beechat's crate, which `be99eadb` replaced with retinue; retinue needs
no `protoc`.
