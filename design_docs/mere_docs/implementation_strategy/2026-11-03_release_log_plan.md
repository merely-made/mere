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

Links: area publishing · draws on 2026-10-29_own_device_serving_brief · draws on 2026-10-28_site_publication_plan · draws on 2026-10-28_murm_runtime_plan · plans R1–R9 · summary one release primitive over every carrier, built in lanes

### release_log D2: ordinary sites take either authority, author's choice (2026-11-03)

**2026-11-03 · new.** Asked whether an ordinary site's releases are vouched for
by a site log, a solo Moot, or both with the author choosing, Mark chose both:

> 3, but what's the redundancy there?

The redundancy is in verification, not in anything else. Both paths share the
release body, content references, carriage, holders, the resolver and the
reader-history store. What doubles: two ways to decide the current release (a
log chain's head versus Gemot's collection fold), two refusal matrices
(tamper, rollback, equivocation or conflict, withdrawal) and a lineage rule for
an author who moves a site from a log to a Moot. R2 keeps that to one reader
history with two authority adapters, and the Moot's first selection names the
log's last release.

Links: area publishing · amends release_log D1 · plans R2 · summary ordinary sites use a site log or a solo Moot; one reader history, two authority adapters

### release_log D3: three address tiers, unassociated first (2026-11-03)

**2026-11-03 · new.** Asked which key roots a site's address, Mark:

> you want all three; root, associated, unassociated addresses. unassociated first and foremost is the most secure and can be associated in a layer above at convenience, but deriving disposable personas from root for permission management is a legitimate benefit when you need to manage other people's identities without owning them (kids, biz, orgs, hierarchy). anonymity is important and manageable identity is important.

Unassociated keys share no derivation with any root, and association is a
signed claim added above them when the holder wants it. Associated keys are
derived from a root as disposable personas, so a parent, business or
organisation can manage permissions over identities it doesn't own. Root
addresses stay available. Unassociated is the default.

Links: area identity · plans R2 · plans R7 · summary root, associated and unassociated addresses; unassociated is the default and association is a layer above

### release_log D4: an HTTP compatibility layer is required, and must not reveal the publisher (2026-11-03)

**2026-11-03 · new.** Asked whether a read-only HTTP gateway is in scope, Mark
chose "later, not now", adding:

> but it is absolutely necessary. in a secure manner. we could potentially even allow people to enable serving a compatibility layer so their sites could be parsed by firefox/chrome; that is doable. but like, if it exposes your ip, who would do it? not me. so there must be a secure means of making available sites, without revealing your whole ass

R9 holds it. It runs after holders and anonymity identities exist, and a
publisher's own devices never serve HTTP on their own addresses.

Links: area publishing · plans R9 · summary HTTP compatibility for mainstream browsers, after R5 and R7, never exposing the publisher's address

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
key takes any of D3's three tiers: unassociated by default (no derivation from
any root; association is a signed claim added later), associated (a disposable
persona derived from a root, for managed identities), or root. Ordinary reader
history mirrors P11: remembered head per log key, refusal of known rollback,
equivocation (two operations at one sequence) surfaced as a conflict, an
explicit freshness scope. Per D2 the reader history takes two authority
adapters, the log chain and a solo Moot's collection fold through P10/P11, and
moving a site from a log to a Moot makes the Moot's first selection name the
log's last release. Decide here whether
out-of-order buffering extends Stickleback's `causal.rs` or adopts
p2panda-stream's ingest; record the reason in Findings.

- Done when: a cold reader process holding only a log key verifies a release
  chain and opens the named snapshot; tampered bytes, a forged operation, known
  rollback and equivocation refuse; an older valid chain opens with its
  limited-freshness scope stated; a fork's first release resolves its source;
  the same refusals hold for a solo-Moot site through the second adapter, and a
  log-to-Moot move keeps lineage; an unassociated key verifies with no root in
  scope, and an association claim verifies separately; the record builds with
  no carrier feature.

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

Carrier identities follow D3: an anonymous carrier uses an unassociated key by
default, so an I2P or onion identity never shares a derivation with the plain
identity; associated and root identities stay available where management
matters more than anonymity. This decides the I2P plan's identity fork. Tor as a default-off `Transport` through arti, after checking its
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

### R9. HTTP compatibility layer

Per D4, mainstream browsers read releases without the publisher's address
being exposed. The publisher's own devices never serve HTTP on their own
addresses. Two routes:

- **Onion HTTP.** A holder serves the release over HTTP as a Tor onion service.
  Tor Browser is Firefox-based, so it reads the site with nobody on the path
  seeing either address. Needs R7's Tor carrier.
- **Clearnet holder.** A holder with its own public address (a rented host, a
  friend, an opted-in reader) serves HTTP and acquires releases from the
  publisher only over a relay or anonymous route. Chrome and Firefox read it;
  the holder's address is public and the publisher's never is.

The browser can't verify the log itself, so the gateway states that it, not the
reader, verified the release, and serves the release hash and evidence beside
each page so a capable client can check it. Small-web pages are rendered to
HTML on the holder; static HTML releases are served as they are.

- Needs: R5, R7.
- Done when: a release opens in Tor Browser through an onion holder and in
  Chrome through a clearnet holder; in both runs the publisher's devices make
  no connection that reveals their address to the holder or the reader; a
  tampered release is refused by the gateway, not served.

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
| gateway | R9 | mere (Djinn), murm | holder, anonymity |

Lanes cost about 30 GiB of build cache each against the shared 400 GiB budget,
so two or three run at a time. `record` and `carriage` are independent;
`anonymity` can run beside either.

## Open questions

- **Gemot in ordinary builds.** D2 lets an author pick a solo Moot, and the site
  publication plan says ordinary serving works without Gemot. Read together,
  the solo-Moot adapter has to be an optional feature, so the site-log path
  still builds without Gemot. Confirm that reading.
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
- 2026-11-03: D2 (both authorities), D3 (three address tiers) and D4 (HTTP
  compatibility without exposing the publisher) recorded; R9 added.
