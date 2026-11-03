# Release log brief: one primitive for sites, shares, clips and publications, with carriers as routes

**Date:** 2026-11-03
**Kind:** research brief digesting a chat with Mark (2026-11-03) that ran from
"how do QUIC, Reticulum, I2P, Tor and Noise-over-TCP cohere in one stack" to
"which Beaker Browser features can we build". Claims from the chat were checked
against origin/main of Murm, Eidetic, Gemot, Knot, Turnstone and Mere the same
day; corrections are in §6.
**Status:** direction. The implementation strategy is
[release_log_plan](mere_docs/implementation_strategy/2026-11-03_release_log_plan.md).
**Anchors:**
[own-device serving brief](2026-10-29_own_device_serving_brief.md) (Beaker's
post-mortem, serving across your devices),
`knot-editor/design_docs/2026-10-29_knot_writing_workspace_and_mesh.md`
("Mesh publication direction", the serving contract and first acceptance gate),
[site publication plan](mere_docs/implementation_strategy/2026-10-28_site_publication_plan.md)
(P10 portable reader verification, P11 remembered reader authority),
[murm runtime plan](mere_docs/implementation_strategy/2026-10-28_murm_runtime_plan.md)
(drops, M12 carriage),
`murm/design_docs/2026-10-28_transport_consolidation_plan.md` (what Murm owns),
`murm/design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md` (the
setup gate), `murm/design_docs/2026-11-02_i2p_sam_seam_plan.md` (the identity
fork), `gemot/design_docs/moothold_docs/implementation_strategy/2026-11-01_moot_publication_plan.md`
(governed publication and hosting records),
`turnstone/design_docs/2026-07-28_turnstone_peer_web_reframe.md`.

## 1. The question as Mark framed it

Mark's first framing was five carriers, each anchoring a vertical, with the
p2panda crates as three layers across all of them:

| Vertical | Carrier | For |
|---|---|---|
| Co-op | QUIC (iroh) | Live shared work with known peers |
| Mesh broadcast | Reticulum | Announce and propagate across the mesh, off-grid included |
| Open / closed submesh | Tor / I2P | Tor reaches the open internet anonymously; I2P is a closed overlay with no exits |
| Encrypted plain network | Noise over TCP, or Reticulum over TCP | Ordinary networks without iroh's QUIC, relays or discovery |

Layers: **reach** (discovery, iroh, sync), **data** (blobs, encryption, store),
**shared spaces** (spaces, auth, stream). He corrected a first reading that
treated the list as transport buckets: "I was kind of more building verticals."

From the own-device serving brief, the goal those verticals serve: "people
should be able to browse, edit, and publish peer-to-peer sites."

## 2. Where the grid stands

Only the co-op vertical has every layer. Murm's `Transport` trait
(`murm/native/crates/transport/src/transport.rs:52`) has three implementations,
`P2pandaTransport`, `ReticulumTransport` and `MemoryTransport`, but gossip,
LogSync, discovery and blob transfer are p2panda-net or iroh-blobs capabilities.
`ReticulumTransport`'s header says it: "Sync (gossip / RBSR / LogSync) and blob
transfer remain iroh-only; this is the bilateral stream lane only."

| | Reach | Data | Shared spaces |
|---|---|---|---|
| Co-op / iroh | p2panda-net discovery, endpoint, LogSync, gossip | store, encryption; blobs via iroh-blobs | Gemot auth, Stickleback group crypto, OCapN live layer |
| Reticulum | retinue announces and address book; no sync; session lane carries posts | store, encryption; no blob transfer (one Retinue response segment is 1,048,575 bytes) | works as data, but nothing delivers missing history |
| I2P | only as a Reticulum interface (truchman SAM, optional embedded emissary) | as Reticulum | as Reticulum |
| Tor | nothing; "Arti is deferred" | — | — |
| Noise / TCP | `NoiseListener` exists but is not a `Transport` | store, encryption | as Reticulum |

The data layer already coheres: every vertical speaks p2panda-core operations,
and ingest is content-addressed and idempotent. Admission partly coheres:
`transport::security` runs natively on iroh and over Noise elsewhere, and the
session lane, Knot publishing and OCapN use it.

## 3. The primitive: a release in a log

p2panda is a signed append-only log per author, each operation identified by
its hash. That is the shape Beaker's Hypercore had. One primitive can carry a
site, a shared folder, a clip and a moot publication:

- **A release** is an operation whose body names content by hash: a
  `PublishedSnapshotV1` digest for a site (snapshot V1 stays unchanged, as the
  site publication plan requires), or a blob manifest for a folder or clip. It
  also carries provenance, an optional `fork_of` release hash and advisory
  provider hints.
- **A release log** is rooted at a key. The header's sequence number and
  backlink order releases and expose rollback and equivocation, which is the
  thing `PublishedSnapshotV1.version` (a schema version) can't do.
- **Seeding** is holding the log and its blobs. A holder can serve a release
  but can't forge one, so own devices, friends and rented mirrors are all
  holders.
- **Forking** starts a new log whose first release names its source.
- **Private sharing** seals the payload; a low-trust holder seeds ciphertext
  and only admitted readers open it. Stickleback's protected drops already have
  this shape.
- **Unlisted sharing** ("share a folder") is a release whose address carries a
  read capability.
- **A clip** is a small release with provenance from the engine that captured it.

Carriers become routes to a release, not features of it. A reader trusts an
anchor it already holds and verifies what any route brings.

### Two release authorities

The chat assumed one. The code has two, and both stay:

- **Governed:** a Moot collection selection. Gemot owns it. The site publication
  plan's P10 (`verify_reader_evidence`) lets a cold reader verify a site from a
  founder/collection anchor plus a public Gemot native drop, and P11
  (`ReaderHistory`) remembers authority and refuses known rollback. Gemot's
  Moot publication plan owns hosting commitments. The mesh record's rule holds:
  don't replace governed selection with a looser public envelope.
- **Ordinary:** an author's own site, which must work "without Gemot
  membership, paired devices or a moot replica" (site publication plan,
  Constraints). This is where nothing self-authenticating exists today. The
  release log fills this gap.

Both reach a reader as evidence plus bytes, and both should share the
reader-history shape P11 established: remembered state per trust anchor,
refusal of known rollback, an explicit "valid under supplied history" scope.

## 4. Addresses, discovery and holders

**Addresses.** Turnstone already parses `knot://<64-hex root>/<path>`
(`turnstone/src/knot_authoring/parse_place_held_address.rs`) and recognises
`mere://`. Extending that grammar beats inventing one: the host is a release-log
key (or a Moot collection anchor for governed sites), the path names a page or
file, and an optional release hash pins a version. Moots and co-op sessions get
openable addresses the same way.

**Discovery gives hints; the log gives authority.** A resolver tries one adapter
per carrier:

- p2panda-net on a topic derived from the log key;
- Reticulum announces whose destination name carries the log key, filtered by
  name (a destination hash can't be computed from a key alone, so holders are
  learned from their announces);
- I2P and onion destinations listed as provider hints in the log.

Discovery adapters are Murm's (transport consolidation plan, Boundary). A hint
never overrides what the log says, and advertising availability copies no
content and grants nothing (site publication plan, Constraints).

**Relays and holders are separate pools.** The mesh record already says "a
transport relay alone is not such a holder."

| Relay | Who still sees your address |
|---|---|
| iroh relay with direct paths disabled (whether iroh 1.0 exposes that setting is unchecked) | the relay operator, not the reader |
| Reticulum transport node | the next hop on your path |
| I2P tunnel or Tor onion | nobody on the path |

Holder pools store releases and acknowledge custody: enrolled own devices first,
then friends, then rented hosts. A personal mesh is a private space whose members
are your own devices; they replicate your sealed vault and your public releases,
relay for each other, and hand serving off as they wake and sleep. Drop receipts
(`stickleback/src/receipt.rs`) deliberately carry no peer identity, so a custody
acknowledgement is a separate signed statement bound to the holder's session.

## 5. Providers, frames and publishers

Mark's point from the chat: Turnstone is the content provider. Its engines
(Genet, the small-web lanes, Scry) capture things. Knot is an authoring frame
and a publisher. That differs from Knot supplying panes to a host. In release
terms, each content source emits releases with provenance, any frame can open
them, and any publisher can release them. Knot's named-recipient gate
(`knot-editor/design_docs/2026-11-02_publishing_protocol_plan.md`, O2) becomes
an unlisted release to one reader.

Static HTML sites fit the release format as content. Dynamic applications come
through mods and contributions, which keeps the own-device serving brief's
ruling: "Native hosts and contributions, not injected web APIs."

## 6. Corrections to the chat

- **p2panda-stream is not the missing ordering logic.** Stickleback already owns
  causal bookkeeping (`causal.rs`: frontiers, topological order, pending and
  missing-parent diagnostics) and the policy-before-insert processor. What the
  off-iroh verticals lack is carriage: something that fetches missing history.
  Whether to adopt p2panda-stream's out-of-order ingest
  (`p2panda-stream/src/ingest/ooo.rs` in the fork) or extend `causal.rs` is a
  choice to make with the code, not a given.
- **Store-and-forward carriage already has a format.** Native drops are
  "transport-independent", with plain and protected variants. Murm runtime M12
  owns "Iroh and Retinue carriage of drops"; the release log rides that rather
  than adding a parallel envelope.
- **Freshness is solved for governed sites, not ordinary ones.** P10 and P11
  landed 2026-10-29 and 2026-10-30.
- **Snapshot size vs Reticulum.** A snapshot V1 can encode to 24 MiB; one
  Retinue response segment is 1,048,575 bytes. Off-iroh blob carriage needs
  hash-verified chunks or Retinue's multi-segment transfer (Knot small-web O2).
- **p2panda-blobs exists in the fork** but nothing uses it; iroh-blobs stays the
  iroh path.

## 7. Tensions

- **Identity vs anonymity.** `ReticulumTransport` derives its Reticulum identity
  from the same Ed25519 key, so a verified announce carries the `PeerID`. That's
  what lets the plain verticals cohere, and it is exactly what an I2P or onion
  identity must not do. A persona's anonymous identities need their own
  derivations, and a release log key shouldn't be the persona root either. The
  current `knot://<root>` form exposes the Personae root. The I2P plan's open
  identity fork sits here.
- **Fork patch cost.** 22 build roots repeat the mark-ik/p2panda patch table.
  Adopting p2panda-spaces adds a row everywhere. Upstreaming the seams or
  publishing the fork under fork names would remove rows; neither is decided.
- **Spaces stability.** p2panda-spaces is the natural owner of group membership
  and key agreement under Gemot's governance meaning (the 2026-10-29 stack
  ownership audit says to evaluate it), but its API is unstable before v1.
- **Footprint rules.** Carriers stay opt-in, Knot builds offline, and default
  builds compile no C (stack modularity direction). The release record and
  reader history belong in the data layer, buildable with no carrier; arti, if
  it comes, stays behind a feature and its storage dependencies need checking
  against the no-C rule.
- **Ordinary browsers.** Existing small-web clients can read the protocols Knot
  serves, but Chrome can't read Gemini any more than it could read `hyper://`.
  Whether a read-only HTTP gateway belongs here is open.

## 8. What Beaker had, mapped onto releases

| Beaker | Here |
|---|---|
| Make a site in the browser | Knot authoring in Turnstone; small-web formats, static HTML as content |
| `hyper://key` | `knot://<log key>/<path>[@release]` |
| Seeding | holders hold the log and blobs; custody acknowledgements |
| Version history | log sequence; Knot checkpoints locally |
| Fork | a log whose first release names `fork_of` |
| Share a folder | unlisted release with a capability address |
| Multi-writer | governed sites through Gemot; space-rooted logs if spaces is adopted |
| Web app APIs | mods and contributions instead |
| Homebase | Djinn resident as a holder |
