# Release log: carrier-neutral sites, shares and clips

**Status:** 2026-11-03: plan. Nothing implemented. Design:
[release_log_brief](../../2026-11-03_release_log_brief.md).

**For:** one primitive, a release in a signed p2panda log, that a site, an
unlisted share, a clip and a moot publication all use, carried over any Murm
carrier and verified by a reader against an anchor it already holds. It closes
the Beaker gaps the 2026-11-03 chat listed: a stable site address, seeding by
holders while the author is offline, forks, link sharing, and the same features
over Reticulum, I2P, Tor and Noise-over-TCP as over iroh.

Not here, and cited instead of repeated:
- governed publication, hosting commitments and historical authority:
  `gemot/design_docs/moothold_docs/implementation_strategy/2026-11-01_moot_publication_plan.md`;
- ordinary serving, the Djinn publication host, P10/P11 reader verification:
  [site_publication_plan](2026-10-28_site_publication_plan.md);
- drop codec, retention and drop carriage (M12):
  [murm_runtime_plan](2026-10-28_murm_runtime_plan.md);
- endpoint authority and the T1–T6 consolidation:
  `murm/design_docs/2026-10-28_transport_consolidation_plan.md`;
- the setup gate and OCapN: `murm/design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md`;
- the I2P identity fork and SAM interop: `murm/design_docs/2026-11-02_i2p_sam_seam_plan.md`;
- Knot's serving contract and first acceptance gate:
  `knot-editor/design_docs/2026-10-29_knot_writing_workspace_and_mesh.md`.

Receipts from this plan's runs go to
`archive_docs/receipts/2026-11-03_release_log_plan_receipts.md`, never into this
plan (DOC_POLICY §8).

## Decisions

### release_log D1: write the brief and strategy, with lanes (2026-11-03)

**2026-11-03 · new.** Mark:

> can you create a design brief and implementation strategy for this? lanes are welcome

> the entire discussion seems quite relevant as context if you can

Links: area publishing · draws on 2026-10-29_own_device_serving_brief · draws on 2026-10-28_site_publication_plan · draws on 2026-10-28_murm_runtime_plan · plans R1–R8 · summary one release primitive over every carrier, built in lanes

## Constraints

- **Two release authorities.** Governed releases are Gemot collection
  selections and keep P10/P11's verification and remembered history. Ordinary
  releases are the new site log and must work with no Gemot membership, paired
  device or moot replica. A release log never replaces governed selection with
  a looser envelope (Knot mesh record).
- **Snapshot V1 unchanged.** A release names a `PublishedSnapshotV1` digest; it
  doesn't add fields to it. The schema version is never a release counter.
- **Ownership.** Release record and reader history: Eidetic/Stickleback
  (mechanics) and `mere-site` (site profile). Carriage, discovery adapters,
  relays and route diagnostics: Murm. Publication host and holder lifecycle:
  Djinn. Release intent and product wording: Knot. Content capture and
  provenance: Turnstone's engines. Governance meaning: Gemot.
- **Footprint.** The record, reader history and resolver core build with no
  carrier. Each carrier adapter is a default-off feature. Default builds compile
  no C. Knot's offline profile stays carrier-free.
- **Hints are not authority.** Provider hints and announces route; the log
  decides. Advertising availability copies no content and grants nothing.
- **Licence.** Reticulum adapters call retinue's API; no RNS-derived code
  leaves retinue, outrider, tulle or radio-hand.
- **No injected web APIs.** Dynamic applications arrive as mods and
  contributions.

## Phases

### R1. p2panda fork posture

Adopting crates the family doesn't link yet (p2panda-spaces, possibly
p2panda-blobs) adds a row to the patch table repeated in 22 build roots. The
posture is Mark's call: upstream the seams (address book, shutdown drain,
admitted ALPN handlers, gossip watcher refresh, dalek 3), publish the fork under
fork names as Bole, Vano and Didgeridoo are, or keep the personal fork.
R2–R6 use only crates already in the table (core, store, sync, encryption,
auth, stream, net) and don't wait on this.

- Done when: the posture is recorded as a decision here, and R8 knows whether
  adoption adds rows.

### R2. Release record and reader history

A release operation body: kind (site, share, clip), content reference
(snapshot V1 digest or blob manifest root), provenance, optional `fork_of`
release hash, optional provider hints, optional sealed-payload marker. The log
is keyed by a release-log key derived from Personae per site, not the persona
root. Ordinary reader history mirrors P11: remembered head per log key,
refusal of known rollback, equivocation (two operations at one sequence)
surfaced as a conflict, an explicit freshness scope. Decide here whether
out-of-order buffering extends Stickleback's `causal.rs` or adopts
p2panda-stream's ingest; record the reason in Findings.

- Done when: a cold reader process holding only a log key verifies a release
  chain and opens the named snapshot; tampered bytes, a forged operation, known
  rollback and equivocation refuse; an older valid chain opens with its
  limited-freshness scope stated; a fork's first release resolves its source;
  the record builds with no carrier feature.

### R3. Carriage over any `Transport`

- LogSync as a Stickleback session over a framed bidirectional stream, so Murm
  can hand it a stream from any carrier. p2panda-net stays the native iroh path.
- Drops carried over `Transport` for carriers without sync; this is murm
  runtime M12's item and lands there.
- Hash-verified chunk fetch over `Transport`, chunks sized under one Retinue
  response segment (1,048,575 bytes), so a 24 MiB snapshot moves over
  Reticulum.
- Noise over TCP as a `Transport` implementation, so services written against
  `Transport` can choose it.

- Done when: two in-process peers converge the same release log over
  `P2pandaTransport`, `ReticulumTransport` and Noise/TCP; a 24 MiB snapshot
  transfers over the Reticulum transport with every chunk verified and a
  tampered chunk refused; all three go through the `transport::security` gate.

### R4. Addresses and resolver

Extend Turnstone's `knot://<64-hex>/<path>` grammar: the host is a release-log
key or a Moot collection anchor, and `@<release hash>` pins a version. One
parser, shared by Turnstone and Knot. A Murm resolver tries one discovery
adapter per enabled carrier: a p2panda topic derived from the key, Reticulum
announces whose destination name carries the key, and I2P or onion destinations
read from provider hints. Moots and co-op sessions resolve through the same
grammar.

- Done when: a reader holding only an address finds a holder on each enabled
  carrier in-process and verifies through R2; a hint pointing at a holder with a
  different log refuses; the governed form resolves to P10/P11 verification.

### R5. Holders, the personal mesh and pools

A holder syncs a log, retains its blobs under an explicit storage policy,
serves on its enrolled carriers and signs a custody acknowledgement for an exact
release, bound to its session (drop receipts carry no peer identity by design).
The personal mesh is a private space of the owner's devices that replicates the
sealed vault and public releases and hands serving off between devices. Relay
pools and holder pools are one stack setting with a visible privacy tier per
route.

- Done when: the Knot mesh record's first acceptance gate passes in-process
  (A releases, B holds and survives restart, A goes away, cold reader C with
  only the address reads R), once over iroh and once over a non-iroh carrier;
  publishing reports "replication pending" until the configured holders
  acknowledge.

### R6. Provider, frame and publisher contract

Turnstone's engines emit clips as releases with provenance; Knot opens a
release as a frame and publishes. Knot's named-recipient gate (publishing
protocol O2) becomes an unlisted release with a capability address. "Share a
folder" is an unlisted release of a blob manifest.

- Done when: a Turnstone clip becomes a release, Knot opens and republishes it
  with provenance intact, and a folder share opens by address for a second
  persona and refuses for a third.

### R7. Anonymity identities and Tor

Per-persona, per-carrier derived identities so an I2P or onion identity never
shares a key with the plain identity; this decides the I2P plan's identity
fork. Tor as a default-off `Transport` through arti, after checking its
storage dependencies against the no-C rule. Tor's open-internet role also
reaches fetch (netfetcher, Errand) through the same feature.

- Done when: no announce, hint or handshake on an anonymous carrier carries a
  key that links to the persona's plain identity; a release resolves and
  verifies through an onion or I2P holder.

### R8. p2panda-spaces evaluation in Gemot

Run the stack ownership audit's deciding test: generic group transitions and
key agreement through p2panda-spaces against Gemot's `GroupCrdt` use and
Stickleback's group session, with Gemot keeping constitution and authority
interpretation. Space-rooted release logs for multi-writer ordinary sites
follow from the answer.

- Needs: R1.
- Done when: the evaluation's result is recorded in Gemot's plan and here,
  with the reason.

## Lanes

Each lane is one worktree in `Code/worktrees/<repo>-release-<lane>` and one
build-storage lane `release-log-<lane>`, through the wrapper only. No bare
`git stash`; WIP commits instead. No attribution trailers. Each lane merges to
main, pushes and removes its worktree and branch. A Mere change forces a family
repin.

| Lane | Phases | Repos | Needs |
|---|---|---|---|
| record | R2 | eidetic (Stickleback), mere (`mere-site`) | — |
| carriage | R3, R4 resolver | murm, eidetic | — (M12 in murm runtime plan) |
| holder | R5 | mere (Djinn), murm | record, carriage |
| frames | R4 grammar, R6 | turnstone, knot-editor | record |
| anonymity | R7 | murm (retinue I2P, arti), Personae custody | — |
| spaces | R8 | gemot | R1 |

Lanes cost about 30 GiB of build cache each against the shared 400 GiB budget,
so two or three run at a time. `record` and `carriage` are independent;
`anonymity` can run beside either.

## Open questions

- **Ordinary-site authority.** A site log (R2 as written), or a solo Moot whose
  founder is the author so P10/P11 apply unchanged? The solo Moot reuses landed
  code but makes Gemot a dependency of ordinary serving, which the site
  publication plan's constraint rules out.
- **Release-log key.** Derived per site from Personae (R2 as written), or the
  persona root that `knot://` uses today?
- **HTTP gateway.** Is a read-only HTTP gateway for ordinary browsers in scope?
- **iroh relay-only.** Does iroh 1.0 let an endpoint withhold direct addresses
  so a reader sees only the relay?

## Findings

- 2026-11-03: `ReticulumTransport` is the bilateral stream lane only; sync and
  blobs are iroh-only (`murm/native/crates/transport/src/reticulum_transport.rs`, header).
- 2026-11-03: `p2panda-sync`'s `Protocol` works over any `Sink`/`Stream` and has
  no iroh dependency (fork rev `1f69c9b`, `p2panda-sync/src/traits.rs:15`).
- 2026-11-03: Stickleback already owns causal bookkeeping
  (`eidetic/crates/stickleback/src/causal.rs`), the policy-before-insert
  processor (`processor.rs`), plain and protected drops (`drop.rs`) and
  identity-free drop receipts (`receipt.rs`).
- 2026-11-03: `NoiseListener` runs Noise over plain TCP but is not a
  `Transport` (`murm/native/crates/transport/src/noise.rs`).
- 2026-11-03: Turnstone parses `knot://<64-hex Personae root>/<path>` and
  refuses other hosts (`turnstone/src/knot_authoring/parse_place_held_address.rs`).
- 2026-11-03: snapshot V1 encodes up to 24 MiB (site publication plan,
  Constraints); one Retinue response segment is 1,048,575 bytes (Knot small-web
  plan O2).
- 2026-11-03: the fork carries p2panda-spaces and p2panda-blobs; neither is
  used outside a cleromancy manifest.

## Progress

- 2026-11-03: plan written with the brief; decision D1 recorded.
