# Release log: carrier-neutral sites, shares and clips

**Status:** 2026-11-05: in progress. R2's first slice and R3's stream-sync
slice landed; R1 published; R7, R4's grammar and R6's first slice landed;
consumers move to `p2mere-*` in a family repin (see Findings, "Wave 2"). Sharing semantics
refined in D9–D11 and R6. Phases rescoped 2026-11-03
after a redundancy pass over Eidetic, Murm, Gemot, Dramatis, Armillary, Errand,
Distillery and Mere (Findings, "Redundancy pass"). Design:
[release_log_brief](../../2026-11-03_release_log_brief.md).

**For:** one primitive, a release in a signed p2panda log, that a site, an
link share, a clip and a moot publication all use, carried over any Murm
carrier and verified by a reader against an anchor it already holds. It closes
the Beaker gaps the 2026-11-03 chat listed: a stable site address, seeding by
holders while the author is offline, forks, link sharing, and the same features
over Reticulum, I2P, Tor and Noise-over-TCP as over iroh.

Not here, and cited instead of repeated:
- governed publication, hosting promises and historical authority:
  `gemot/design_docs/moothold_docs/implementation_strategy/2026-11-01_moot_publication_plan.md`;
- ordinary serving, the publication host, P10/P11 reader verification, public
  binding (P3, P4): [site_publication_plan](2026-10-28_site_publication_plan.md);
- drop codec, retention and drop carriage over iroh and Retinue (M12):
  [murm_runtime_plan](2026-10-28_murm_runtime_plan.md);
- device pairing: [device_pairing_plan](2026-10-28_device_pairing_plan.md);
- supervision, the scheduler (phase E), DNS-SD (F3) and the remote-listener rule
  (F2): [family_update_resident_plan](2026-10-28_family_update_resident_plan.md);
- distinct per-persona NodeIds and discovery off for non-main personas:
  [browser_carrier_plan](2026-10-28_browser_carrier_plan.md);
- the p2panda-spaces deciding test: the stack ownership audit, item R25
  (`design_docs/mere_docs/research/2026-10-29_stack_ownership_audit/REPORT.md`);
- endpoint authority: `murm/design_docs/2026-10-28_transport_consolidation_plan.md`;
- the setup gate: `murm/design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md`;
- the I2P identity fork and SAM interop: `murm/design_docs/2026-11-02_i2p_sam_seam_plan.md`;
- Knot's serving contract and first acceptance gate:
  `knot-editor/design_docs/2026-10-29_knot_writing_workspace_and_mesh.md`;
- sleep hand-off and "last one out": [own-device serving brief](../../2026-10-29_own_device_serving_brief.md) §2.
- cross-client application profiles, applets, extension-only smolweb retrieval
  and contact/service trust: [portable application experience](../../2026-11-04_portable_application_experience_brief.md).
- Copy link, the local browser companion and product receiving gates:
  [application experience plan](2026-11-04_application_experience_plan.md).
  Its S1 works with existing references; S2/S3 consume this plan's qualified
  R2/R4/R6 mechanisms rather than introducing another release/share codec.

Receipts from this plan's runs go to
`archive_docs/receipts/2026-11-03_release_log_plan_receipts.md`, never into this
plan (DOC_POLICY §8).

## Decisions

- **D1, write the brief and strategy, with lanes (2026-11-03):**

**2026-11-03 · new.** Mark:

> can you create a design brief and implementation strategy for this? lanes are welcome

> the entire discussion seems quite relevant as context if you can

Links: area publishing · plans release-log · summary one release primitive over every carrier, built in lanes

- **D2, ordinary sites take either authority, author's choice (2026-11-03):**

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

Links: area publishing · amends release-log D1 · plans release-log · summary ordinary sites use a site log or a solo Moot; one reader history, two authority adapters

- **D3, three address tiers, unassociated first (2026-11-03):**

**2026-11-03 · new.** Asked which key roots a site's address, Mark:

> you want all three; root, associated, unassociated addresses. unassociated first and foremost is the most secure and can be associated in a layer above at convenience, but deriving disposable personas from root for permission management is a legitimate benefit when you need to manage other people's identities without owning them (kids, biz, orgs, hierarchy). anonymity is important and manageable identity is important.

Unassociated keys share no derivation with any root, and association is a
signed claim added above them when the holder wants it. Associated keys are
derived from a root as disposable personas, so a parent, business or
organisation can manage permissions over identities it doesn't own. Root
addresses stay available. Unassociated is the default.

Links: area identity · plans release-log · summary root, associated and unassociated addresses; unassociated is the default and association is a layer above

- **D4, an HTTP compatibility layer is required, and must not reveal the publisher (2026-11-03):**

**2026-11-03 · new.** Asked whether a read-only HTTP gateway is in scope, Mark
chose "later, not now", adding:

> but it is absolutely necessary. in a secure manner. we could potentially even allow people to enable serving a compatibility layer so their sites could be parsed by firefox/chrome; that is doable. but like, if it exposes your ip, who would do it? not me. so there must be a secure means of making available sites, without revealing your whole ass

R9 holds it. It runs after holders and anonymity identities exist, and a
publisher's own devices never serve HTTP on their own addresses.

Links: area publishing · plans release-log · summary HTTP compatibility for mainstream browsers, after R5 and R7, never exposing the publisher's address

- **D5, tailor the p2panda fork, take from upstream as needed (2026-11-03):**

**2026-11-03 · new.** On the fork posture R1 asked about, Mark:

> we can edit the fork to tailor it to our needs; take what's needed from upstream, too.

R1 is settled: the mark-ik/p2panda fork carries what the family needs and
pulls upstream work when it's wanted. Adopting a crate the family doesn't link
yet adds a row to each root's patch table; that cost is accepted. R8 no longer
waits.

Links: area dependencies · plans release-log · summary keep and tailor the p2panda fork, merging upstream as needed

- **D6, the solo-Moot adapter is optional (2026-11-03):**

**2026-11-03 · refines release-log D2.** Asked to confirm that the solo-Moot
path is an optional feature so the site-log path builds without Gemot, Mark:

> agreed with the optional solo-moot of it; not every site's a community!

Links: area publishing · amends release-log D2 · plans release-log · summary the solo-Moot adapter is a feature; the site-log path builds without Gemot

- **D7, publish the fork as `p2mere-*` (2026-11-03):**

**2026-11-03 · refines release-log D5.** Told that tailoring the fork under
upstream names adds a patch row per adopted crate to every root, Mark asked:

> So like, I should rename it and publish fork crates instead? And cherry pick from upstream on an ongoing basis, kinda like cambium/meristem?

On the name he first offered "mere2panda-*, or m2p-*" (the bare `m2p` is an
unrelated Markdown-to-PDF crate), asked for better options and animals, then:

> how about mere2p, or p2mere?

> alright, sounds good

`p2mere-*` was chosen. It mirrors upstream (`p2panda-core` becomes
`p2mere-core`), every name is free on crates.io and has one hyphen (checked
2026-11-03). The fork follows the `genet-parley` precedent (new package names,
library names unchanged) and Meristem's (a provenance ledger with selective
upstream intake). R1 becomes the migration.

Links: area dependencies · amends release-log D5 · plans release-log · summary rename and publish the p2panda fork as p2mere-*, keep library names, take upstream selectively with a ledger

- **D8, compatibility ranks below coherence across transports (2026-11-03):**

**2026-11-03 · new.** Mark:

> staying compatible is a priority, but ensuring compatibility with and coherent usage of other transports than iroh is a bigger and stack-defining priority

Wire compatibility with upstream p2panda peers is kept where it costs nothing.
Where it conflicts with running the same sync, ordering and group machinery
over Reticulum, I2P, Tor and Noise/TCP, the carrier-neutral path wins. Tailored
changes are additive where possible, so iroh peers still speak upstream's
protocols.

Links: area dependencies · plans release-log · summary upstream wire compatibility is kept where free; coherent use over non-iroh transports wins any conflict

- **D9, the recipient controls their copy or access (2026-11-04):**

**2026-11-04 · refines release-log D1.** Asked whether the default share should
be a copy the recipient controls or ongoing access to the author's resource,
Mark answered:

> a copy/access they control. for ongoing access to the resource you yourself are working with, you need like co-op (realtime access to the host's mere) or for the datalake to be served to them as a moot or the relevant data as a static site. that's what intuitively makes sense to me, anyway

Do not make a recipient's retained edition depend silently on continued access
to the author's working state. Live co-op, governed exposure and site
publication are explicit modes. An independently retained export and a replica
that remains governed by a source namespace have different lifecycles.

Links: area publishing · amends release-log D1 · plans release-log · summary recipient-controlled copies or access are the default; ongoing working-state access is explicit

- **D10, create a portable link before a contact flow (2026-11-04):**

**2026-11-04 · refines release-log D9.** Mark:

> create a link, first and foremost. my contacts exist in tons of places, at the os level, with various services, and being able to share data across 'em to the correct address is valuable. so a portable link beats a contact flow.

Copy link is the primary action. OS share sheets and service-specific delivery
can use the link without requiring contact import. Portability does not imply
public read access, bearer authority or permission to delegate a grant.

Links: area publishing · amends release-log D9 · plans release-log · summary portable links precede contact selection; delivery does not determine access authority

- **D11, constrain owned access, not somebody else's address (2026-11-04):**

**2026-11-04 · refines release-log D10.** Mark:

> if it's your resource and you're granting access to it, you can impose identity/distribution constraints. if you're just passing a reference or an address that the person then has, you don't control them being able to access that address, whoever controls that address does. i thus can't say, don't forward this in the latter case, but i can say only you can access my mere and see the site i've annotated here. if i export and serve my annotated site, annotations and all, then my choice is whether i'm serving it to one person (like sharing files in the beaker browser example) or if i'm broadly sharing it to a private subset of kin/kith/etc. or just public. can't take back a served resource, though you can certainly stop serving it.

A named-recipient grant is not an unlisted bearer share. Forwarding its link
does not confer the recipient's rights. An owner can stop serving and change
future authorized access; independently retained plaintext cannot be erased by
that act. A replica that remains within a synchronized domain follows that
domain's separately specified retention rules. Willow is relevant lifecycle
prior art, not an adoption decision; see the
[experience brief](../../2026-11-04_portable_application_experience_brief.md#distributed-deletion-and-willow).

Links: area publishing · amends release-log D10 · plans release-log · summary owned resources can restrict audience and delegation; references inherit source authority and independent copies are not remotely recalled

- **D12, forward secrecy where it fits, bounded durability elsewhere (2026-11-05):**

**2026-11-05 · new.** The R8 evaluation found p2panda-spaces offers only the
durable data scheme and that the family's `forward_secure_messages` profile is
a label with no mechanism behind it. Asked to compare durable data with and
without forward secrecy, Mark:

> Compare durable data with and without forward secrecy. Me personally, i think it makes the network more trustable

Then, on the per-lane recommendation:

> yeah, record D12 and wire it in

Conversations (Murm posts, co-op chat, direct messages) default to
forward secrecy through p2panda-encryption's `message_scheme` ratchet, so a
stolen device or leaked key doesn't open past messages. Shared and published
data (sealed releases, moot collections, shares) keeps the durable
`data_scheme`, because late readers and holders must be able to decrypt it,
with bounded retention: old epoch secrets are forgotten after a set window
through Stickleback's `forget_authorized`, giving forward secrecy at epoch
granularity. Spaces, if adopted, carries the durable side; the ratchet sits
beside it for conversations.

Links: area encryption · plans release-log · summary forward-secure conversations by default; durable shared data with bounded epoch retention

## Constraints

- **Two release authorities.** Governed releases are Gemot collection
  selections and keep P10/P11's verification and remembered history. Ordinary
  releases use a site log, or an optional solo Moot (D2, D6). A release log
  never replaces governed selection with a looser envelope (Knot mesh record).
- **Snapshot V1 unchanged.** A release names a `PublishedSnapshotV1` digest; it
  doesn't add fields to it. The schema version is never a release counter.
- **Ownership.** Release record: Stickleback (mechanics) and Gemot's
  `gemot-site` (`gemot/crates/site-model`, site profile; extracted from Mere and
  Knot and qualified 2026-11-04, gemot `0324688`, mere `f9e8cea64`). The extracted live LogSync session belongs to Murm's `murm-sync`
  (transport consolidation T7); Stickleback retains admission and history.
  The [source homes map](../technical_architecture/2026-10-28_source_homes_map.md#site-and-small-repository-scope-refinement-2026-11-04)
  scopes the general site domain out of both Mere and Knot. Carriage, discovery adapters, relays and route
  diagnostics: Murm. Holder lifecycle: Djinn (`armillary/crates/djinn`) with
  the resident's site and blob owners (`mere/ports/mere-resident`). Release
  intent and wording: Knot. Content capture and provenance: Turnstone's
  engines. Governance meaning and hosting promises: Gemot. Keys: Personae.
- **Footprint.** The record, reader history and resolver core build with no
  carrier. Each carrier adapter is a default-off feature. Default builds compile
  no C. Knot's offline profile stays carrier-free. The site model lives in the
  Gemot repository, but its default build needs no Moot membership; the
  `publication` feature pulls Gemot authority, Personae and Stickleback, so the
  site-log adapter lives in `site-model`'s default features, outside it (D6).
- **Hints are not authority.** Provider hints and announces route; the log
  decides. Advertising availability copies no content and grants nothing.
- **Licence.** Reticulum adapters call retinue's API. Matching announces by a
  key-bearing name, or propagation-node peering, would be RNS-derived logic and
  lands inside retinue or outrider, never in `transport` or Stickleback.
- **No injected web APIs.** Dynamic applications arrive as mods and
  contributions. The experience brief separates ordinary embedded pages from
  a proposed explicitly admitted cooperative applet interface; it grants no
  ambient web-page access to the host.
- **Names.** "Transport" means Murm's `transport::Transport`; netfetcher and
  `script/wire` have their own traits of that name. Release types avoid the
  `Release*` names the family update plan's A4 uses for application updates
  (`ReleaseRefV1`, `ReleaseOfferV1`).

## Phases

### R1. The fork as published `p2mere-*` crates

Per D5 and D7. Today 20 roots repeat seven patch rows pointing crates.io
`p2panda-*` names at the mark-ik fork tag
`mere-p2panda-net-0.7.5-upstream-20261013`, because Cargo applies `[patch]`
only from the root being built. `p2panda-net` is already renamed
(`mere-p2panda-net`; 0.7.2 was published) but keeps two hyphens, against the
house rule.

- Rename every package the family uses to `p2mere-<crate>` (core, store,
  sync, encryption, auth, discovery, stream, net, and spaces and blobs when
  adopted), with internal dependencies pointing at each other. Library names
  stay `p2panda_*`, so consumers change manifests only.
- Keep upstream's MIT OR Apache-2.0 licence, so patches can still be offered
  back.
- Publish in dependency order. Consumers declare the `p2mere-*` versions
  and every root drops its p2panda patch rows. `mere-p2panda-net` is yanked or
  left as a final release pointing at `p2mere-net`.
- A provenance ledger beside the fork records upstream bases, the family's
  patches and anything cut, as `upstream-xilem.md` does for Meristem. Upstream
  work comes in by selective merge or cherry-pick; under D8, a change that
  breaks carrier-neutral use is not taken as is.
- The fork moves from mark-ik to merely-made, recorded in merelyllc.com's
  `ops/org-migration.toml`.

- Done when: no root in `Code/repos` has a `mark-ik/p2panda` patch row; every
  family crate that depended on p2panda resolves `p2mere-*` from crates.io;
  the ledger exists; the fork lives in merely-made.

### R2. Release record and reader history

A release operation body: kind (site, share, clip), content reference
(snapshot V1 digest or blob manifest root), provenance, optional `fork_of`
release hash, optional provider hints, optional sealed-payload marker. Its
log key takes any of D3's tiers (R7 gives the key types).

Reuse:
- Stickleback `causal.rs` for order and equivocation: `CausalError::ForkedSequence`
  (`:122`), `causal_projection` (`:167`), `author_head` (`:321`), pending and
  missing-parent diagnostics. Out-of-order buffering extends it; p2panda-stream
  isn't needed, and Stickleback outside `net` builds with no carrier.
- Stickleback `processor.rs` `OperationPolicy::admit` (`:124`) for the release
  body's policy, `store.rs` `put_blob`/`get_blob` (blake3 `BlobRef`) for content.
- Stickleback `drop.rs` `DropProtector` and protected drops for sealed payloads.
- `gemot-site` `publication/reader/history.rs` (`ReaderHistory`, P11): refactor it
  into one store with two adapters. Its compare-and-write, clock high-water
  mark, capacity bound and corrupt-state refusal carry over. The Gemot adapter
  is today's code behind the optional solo-Moot feature (D6); the site-log
  adapter adds a remembered head per key.
- Gemot `moot/records/collection.rs` (`authorized_collection`, `CollectionFork`)
  is the second adapter. A solo Moot is a Moot whose founder is the author;
  `constitution/mod.rs:11`'s founder-signed first rule needs no helper.
- `gemot-site` `history.rs` `SiteRevision.sequence` is a local checkpoint
  journal and stays separate from the log sequence.

New: the release body, the site-log adapter, and log-to-Moot lineage (Gemot's
collection lineage is same-Moot only; cross-Moot lineage is Gemot P3).

- Done when: a cold reader process holding only a log key verifies a release
  chain and opens the named snapshot; tampered bytes, a forged operation, known
  rollback and equivocation refuse; an older valid chain opens with its
  limited-freshness scope stated; a fork's first release resolves its source;
  with the solo-Moot feature on, the same refusals hold through the Gemot
  adapter and a log-to-Moot move keeps lineage; the site-log path builds
  without Gemot and with no carrier feature.

### R3. Carriage over any carrier

- **LogSync over a stream.** A `murm-sync` session (`murm/native/crates/sync`,
  where T7 moved Stickleback's live sessions) that runs p2panda-sync's
  `Protocol` over a framed bidirectional stream, so any carrier's stream can
  feed it. p2panda-net stays the native iroh path (`murm-sync`'s
  `synced_space.rs`, `joined_space.rs`). Murm's `session_lane.rs` is the proven pattern for gated
  carriage off gossip. Stickleback's `object_stream.rs` caps an object at
  64 KiB, so LogSync framing needs its own bound. This moves catch-up sync,
  not iroh gossip; Reticulum keeps its own announce/propagation mechanisms.
- **Drops and blobs off iroh.** Murm runtime M12 owns drop carriage over iroh
  blobs and Retinue resources, with identical import reports across file, iroh
  and resource. Large content rides that: Retinue's `SegmentedSender` and
  `SegmentedReceiver` (`retinue/src/resource_transfer.rs`) already move
  multi-segment resources past `MAX_SEGMENT_SIZE`, and Stickleback's
  `BlobChunk` and `assemble_chunks` (`drop_io.rs:893`) already verify chunks.
  `ReticulumTransport` uses the reliable stream, not resources, so what's new
  is exposing resource carriage through Murm, not a new chunker.
- **Noise over TCP.** `NoiseListener` stays outside `Transport`, as its own
  ruling says (`noise.rs`, "Deliberately not a Transport": it dials by address
  and owns no discovery). R4's resolver supplies the address, and services
  already receive Noise sessions as `TransportKind::Noise` through
  `AcceptedSession`. Require each new release route to pass the selected
  `transport::security` setup/admission profile. Do not infer that Graphshell's
  WebRTC door already participates: qualifying its challenge/admission against
  the shared setup gate remains a distinct check in the browser-carrier and
  secure-sessions owner plans.

The initial gap inventory also identifies the `ReticulumTransport` header's
RNS 1.3.x claim against Retinue's reported RNS 1.5.7 baseline. Refresh that
header against the selected supplier during Murm's carrier work; this plan
does not repair the header or establish wire compatibility by inspection.

- Done when: two in-process peers converge the same release log over
  `P2pandaTransport`, `ReticulumTransport` and Noise/TCP through the gate; a
  24 MiB snapshot reaches a reader over Reticulum through M12's resource
  carriage with every chunk verified and a tampered chunk refused.

### R4. Addresses and resolver

Extend Turnstone's `knot://<64-hex>/<path>` grammar: the host is a release-log
key or a Moot collection anchor, and `@<release hash>` pins a version. One
parser, shared by Turnstone and Knot. A Murm resolver tries one adapter per
enabled carrier:

- p2panda: `subscribe`, `peers_for_topic` and `set_topics` on a topic derived
  from the key (`p2panda_transport.rs:1033-1159`), with `MunimentAddressBook`.
- Reticulum: extend the ALPN-to-destination-name mapping
  (`reticulum_transport.rs:392`) so holders announce a name carrying the key;
  name filtering on announces lives in retinue.
- I2P and onion: destinations read from provider hints. I2P is reachable only
  as a Reticulum interface today, so a direct I2P stream path is new.
- Noise/TCP: addresses from hints, handed to `NoiseListener::connect_addr`.

LAN DNS-SD stays family update F3. Provider hints follow the same rule as A4's
disposable locators: never trusted.

- Done when: a reader holding only an address finds a holder on each enabled
  carrier in-process and verifies through R2; a hint pointing at a holder with a
  different log refuses; the governed form resolves to P10/P11 verification.

### R5. Holders, the personal mesh and pools

Extend what exists:
- **Serving and retention:** `mere-resident` `resident_site.rs` and
  `moot_site.rs` already retain snapshots under a custody lease and serve them
  with restart and Stop/Remove rules; `resident_blobs.rs` owns blob custody with
  namespaced `BlobLease`s. A holder adds log sync and peer acquisition.
- **Supervision:** Armillary `ResidentTasks` (`close_and_join`, `cancel_and_join`)
  and Djinn `supervise`. Restart and backoff wait on family update phase E.
- **Personal mesh:** `personal_sync_host.rs`, Graphshell's `PersonalGraphReplica`
  and `SyncRoster`, Dramatis's `DeviceRoster`, and the device pairing plan (D1
  landed, D5 open). Vault replication between devices has no plan yet; the
  archived wallet carry layer plan covers carrying a persona.
- **Custody acknowledgement:** new. Gemot's `HostingPromise` is "neither a
  custody claim nor a retrieval observation", and its publication plan keeps
  promise, accepted transfer and demonstrated service as separate facts. The
  acknowledgement is the accepted-transfer fact: Gemot P1 records it for
  governed sites; a holder-signed statement bound to the session records it for
  ordinary ones (drop receipts carry no peer identity by design, `receipt.rs:83`).
- **Mesh holders:** Outrider's LXMF propagation nodes are a Reticulum
  store-and-forward holder; node peering isn't implemented.
- **Pools:** relay and holder pools as one stack setting with a visible privacy
  tier per route. Gemot's
  `moothold_docs/implementation_strategy/2026-11-03_moot_relay_pools_plan.md`
  owns moot-operated relay pools (P0–P2 done, with an I2P host-location privacy
  gate passed for Emissary 0.4.0 and i2pd 3.0.0) and keeps relaying separate
  from content hosting. R5 adds the personal-mesh pool and the setting that
  selects among pools; it doesn't build a second relay service.

- Done when: the Knot mesh record's first acceptance gate passes in-process,
  once over iroh and once over a non-iroh carrier; publishing reports
  "replication pending" until the configured holders acknowledge.

### R6. Provider, frame and publisher contract

The application experience plan owns Copy link and browser/native receiving
integration. This phase owns the release/share authority mechanism it consumes;
it is not a prerequisite for copying an existing external reference.

Turnstone's engines emit clips as releases with provenance; Knot opens a
release as a frame and publishes. Per D9–D11, a portable link can identify a
reference, an independently retained release, or access to an owned resource.
Its audience and lifecycle are explicit; it is not universally a bearer
capability. A folder share names a blob manifest. Knot's named-recipient gate
(publishing protocol O2, owned there) stays identity-restricted rather than
becoming an unlisted bearer release.

Keep resource/edition identity, audience authority, delivery address and
permitted delegation distinct. A link to someone else's site supplies no
authority to restrict that site's readers. An exported annotated edition has
its own publisher/audience choices and source provenance. Grant admission and
encryption need concrete owners; Gemot delegation and private collection
encryption (P3) cover governed cases, not automatically every ordinary share.
No generic capability-address implementation is claimed here.

- Done when: a Turnstone clip opens and republishes in Knot with provenance
  intact; Copy link works without contact import; an ordinary reference keeps
  its source authority; a named-recipient folder link opens for its admitted
  persona and refuses a third persona even after forwarding; an explicitly
  public or bearer share follows its selected audience instead; an independent
  retained copy reopens after the publisher stops serving; edits create fork
  lineage rather than inheriting the author's authority. Live access is
  identified as such and does not promise offline retained-copy behavior.

### R7. Key tiers, anonymous identities and Tor

What each D3 tier maps to:
- **Associated:** Personae `derive_keypair` / `derived_keypair(salt)`
  (BLAKE3-keyed from the master seed) with insigne `DerivedKeyAttestation` as
  the link to the root. Exists.
- **Unassociated:** Gemot's mien already models it as a new unlinked chain with
  zero standing (`persona_chain.rs`). The key is a fresh Ed25519 key held in
  custody outside the master-derived tree. Association added later is a scoped,
  revocable insigne `DelegationCertificate`-style claim, not a
  `DerivedKeyAttestation`, which binds a derivation.
- **Root:** today's `knot://<root>`.

Conflicts to resolve here:
- Murm binds production transports only from a `DerivedKeypair`
  (`p2panda_transport.rs:641` `builder_for`, vault lock ruling 96), and
  `DerivedKeypair::new` is `pub(crate)` in Personae. An unassociated carrier
  key needs a Personae-owned custodied key type that keeps the master
  unreachable.
- The Reticulum identity is derived from the persona key
  (`transport/src/reticulum_transport/keys.rs`, Dramatis `reticulum.rs:62`).
  That's the associated tier; anonymous carriers need the unassociated one.
- The archived unlinkability plan skipped I2P and Yggdrasil for personas. R7
  reverses that for anonymous carriers and decides the I2P plan's identity fork.
- Arti is deferred in the I2P plan. A Tor `Transport` lifts that deferral, behind
  a default-off feature, after checking arti's storage dependencies against the
  no-C rule. For fetch, netfetcher already has the seam: `Stores.transport`
  (`mere/crates/system/fetch/src/handle.rs:147`, "a proxy, an onion or garlic
  lane").

- Done when: no announce, hint or handshake on an anonymous carrier carries a
  key that links to the persona's plain identity; a release resolves and
  verifies through an onion or I2P holder.

### R8. p2panda-spaces evaluation in Gemot

This executes the stack ownership audit's item R25, whose deciding test is
already written: concurrent add and remove, nested devices, key rotation,
offline revoked peers, cold reopen, Muniment without SQLite, durable-data
versus forward profiles. The code under test is Gemot `moot/group.rs`
(`MootGroup` over `GroupCrdt`) and Stickleback `group_session.rs`,
`group_lane.rs`, `group_crypto.rs`. Space-rooted logs for multi-writer
ordinary sites follow from the answer.

- Done when: R25's result is recorded in the audit and Gemot's plan, with the
  reason.

### R9. HTTP compatibility layer

Per D4, mainstream browsers read releases without the publisher's address
being exposed. The publisher's own devices never serve HTTP on their own
addresses. This is a publisher-disclosure requirement with named observers,
not a promise that no intermediary sees any address. Distinct from the
experience brief's retrieval gateway, which fetches smolweb for an extension.
Two publication routes:

- **Onion HTTP.** A holder serves the release over HTTP as a Tor onion service.
  Tor Browser can read it without a clearnet origin endpoint. Its route profile
  must state the endpoint, first-hop and intermediary observations it protects
  against; using an onion address does not establish universal anonymity.
  Needs R7's Tor carrier.
- **Clearnet holder.** A holder with its own public address (a rented host, a
  friend, an opted-in reader) serves HTTP and acquires releases from the
  publisher only over a relay or anonymous route. Chrome and Firefox read it;
  the holder's address is public. The selected route must withhold the
  publisher's direct endpoint from the reader and holder; relays and local
  network observers have their own explicitly stated visibility.

Existing rules this phase has to answer:
- Serving binds loopback only and "public exposure needs its own reviewed
  deployment receipt" (site publication plan, Constraints; P3, P4).
  `GeminiListenerPolicyV1` is loopback and `deny_unknown_fields`, so a public
  listener needs its own policy type.
- Family update F2 keeps remote listeners out of scope "until a consumer
  supplies an admission and disclosure model". R9 is that consumer and supplies
  the model.
- Errand "does not speak HTTP, on purpose" (`errand/src/lib.rs`). The HTTP
  server needs its own owner; Errand's `serve::Source` trait can feed it.
- Gemot's `site-model` `generate` now writes HTML, Gemtext, Scroll and
  Markdown from one source (gemot `c218f90`), so a holder can serve HTML
  generated at publication time for sites authored that way. Translating
  arbitrary small-web pages to HTML is still new; parsers exist (Errand
  `parse/gemtext.rs`, `parse/scrolltext.rs`; Micron in Turnstone and retinue's
  NomadNet layer).

An ordinary browser viewing translated HTML does not itself verify the log.
The gateway states that it verified, using P10/P11's `EvidenceScope` vocabulary,
and serves the release hash and original evidence beside each page so a
capable client can check them. An extension's capable release reader may
verify the original bytes locally; this does not authenticate arbitrary
gateway-translated HTML as those bytes.

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
| fork | R1 | p2panda fork, then every root's manifest | — |
| record | R2 | eidetic (Stickleback), gemot (`site-model`, and the solo-Moot adapter) | — |
| carriage | R3, R4 resolver | murm (`murm-sync`, `transport`, retinue), eidetic | M12 in the murm runtime plan |
| holder | R5 | mere (`mere-resident`), armillary (Djinn), murm | record, carriage |
| frames | R4 grammar, R6 | turnstone, knot-editor | record |
| identity | R7 | dramatis (Personae, insigne), murm, mere (fetch) | — |
| spaces | R8 | gemot, eidetic | fork, plus an upstream intake |
| gateway | R9 | a new HTTP owner, mere (`mere-resident`), armillary | holder, identity |

Lanes cost about 30 GiB of build cache each against the shared 400 GiB budget,
so two or three run at a time. `fork`, `record`, `carriage` and `identity` are
independent of each other. `spaces` waits for the fork and an upstream intake:
upstream integrated spaces into its high-level node crate on 2026-11-04, so
evaluating the 2026-10-13 base would judge an older spaces; the fork's last step touches every
root's manifest, so it lands between other lanes' merges.

## Findings

### Sharing and decision indexing, 2026-11-04

The existing D1–D8 used headings and phase-name fields that Responsa did not
parse. They now use its decision-entry form and mechanical `release-log`
keyword, with the same numbers and owner words; D9–D11 follow in answer
order. The plan's source references remain in its opening list rather than
being misrepresented as decision keys. A current-manifest check alone had not
proved these decisions were present; qualification also requires explicit
lookup of the release-log entries.

The [experience brief](../../2026-11-04_portable_application_experience_brief.md#distributed-deletion-and-willow)
checks Willow's destructive namespace lifecycle against its data-model,
Meadowcap and URI specifications. Those semantics do not follow from a grant
or a precise reference alone, and no storage migration is selected here.

### Carrier source snapshot, 2026-11-03

- 2026-11-03: `ReticulumTransport` is the bilateral stream lane only; sync and
  blobs are iroh-only (`murm/native/crates/transport/src/reticulum_transport.rs`, header).
- 2026-11-03: `p2panda-sync`'s `Protocol` works over any `Sink`/`Stream` and has
  no iroh dependency (fork rev `1f69c9b`, `p2panda-sync/src/traits.rs:15`).
- 2026-11-03: Turnstone parses `knot://<64-hex Personae root>/<path>` and
  refuses other hosts (`turnstone/src/knot_authoring/parse_place_held_address.rs`).
- 2026-11-03: the fork carries p2panda-spaces and p2panda-blobs; neither is
  used outside a cleromancy manifest.
- 2026-11-03, iroh relays: iroh 1.4.1's `Endpoint::builder().clear_ip_transports()`
  removes IP transports, leaving a relay-only endpoint, so a reader sees the
  relay's address and not the publisher's. By default an endpoint also offers
  direct addresses for hole punching, which a peer learns. The relay operator
  (n0 for the default relays) sees the address of each endpoint connected to
  it; a self-hosted relay keeps that in your hands.
- 2026-11-03, redundancy pass, Eidetic and Murm: Stickleback already has causal
  order and fork-at-sequence detection, policy-before-insert, blake3 blob
  storage, protected drops, verified chunk assembly and identity-free receipts.
  Retinue already has multi-segment resources. No release body, remembered
  head, LogSync-over-stream, resolver, custody acknowledgement or pool setting
  exists. `NoiseListener` is not a `Transport` by an explicit ruling; the first
  draft of R3 would have reversed it.
- 2026-11-03, redundancy pass, Gemot, Dramatis and Personae: P11's
  `ReaderHistory` is Gemot-only and becomes the shared store. Personae derives
  every persona from the master, so associated keys exist and unassociated ones
  need a new custodied key type; mien already models the unlinked tier.
  `HostingPromise` is not a custody acknowledgement. R8 restated the audit's
  R25.
- 2026-11-03, redundancy pass, Armillary, Distillery and Errand: Djinn lives in
  `armillary/crates/djinn`; the resident's site and blob owners are the holder
  base. Armillary supervises but has no leases or restart. Distillery's
  inference code isn't in a local repo, and Mere's `distillery` crates
  (Alembic, Athanor) don't overlap. Errand refuses HTTP by design and no
  small-web-to-HTML renderer exists. netfetcher's `Stores.transport` is the Tor
  fetch seam.

### Wave 1, 2026-11-05

- **R2 edition record.** Stickleback `stickleback::edition` (eidetic `4a5b446`,
  `a7a1458`): `EditionBody` (kind, `ContentRef`, provenance, `fork_of`, advisory
  hints, `SealedMarker`) as bounded canonical JSON, `EditionPolicy` on
  `OperationPolicy::admit`, `verify_edition_chain` (signatures, body hashes,
  contiguous sequence and backlinks from the key alone; `ForkedSequence`
  surfaces as `Equivocation`), `EditionWriter` resuming from `author_head`,
  portable evidence and `EditionChain::resolve_fork`. Rules chosen: `fork_of`
  only at sequence 0; a Site edition must carry a snapshot reference.
- **R2 reader history.** gemot-site (gemot `3ed2ba4`, `847b4af`):
  `remembered.rs` is the one store (an `Authority` trait over P11's
  compare-and-write, clock high-water mark, capacity bound and corrupt-state
  refusal) in default features; `site_log.rs` is the site-log adapter;
  `publication/reader/history.rs` is the Gemot adapter, unchanged on disk
  (`site-reader/v1/…`; the site log uses `site-log/v1/…`). Default features
  pull no Gemot authority, iroh, p2panda-net or Murm. Older evidence merges
  into the remembered chain; rollback refuses only when the offered snapshot
  belongs to an earlier edition than the merged head. Equivocation is caught
  only when both branches are held; a cold reader holding one branch can't
  tell it from the truth.
- **R2 open.** Log-to-Moot lineage: `CollectionFork` holds a same-Moot
  `CollectionVersion` parent, so a Moot's first selection naming a log's last
  edition needs a new Gemot field or record, overlapping Gemot P3's cross-Moot
  lineage. Untested: the Gemot adapter's rollback and equivocation refusals
  beyond P10/P11's existing suites, corrupt-state refusal through the site-log
  adapter, and `EditionPolicy` through `OperationProcessor` rather than `admit`.
- **Pin cascade.** A Stickleback change moves eidetic, then murm (whose
  `murm-sync` types appear in Gemot's API), then gemot's root workspace and
  the separate `site-model` and `site-host` workspaces, which carry their own
  locks and patch tables (murm `b6bedc30e21`).
- **R3 stream sync.** murm `418d29a300e` (merged `b6ed4df6189`):
  `murm-sync` `stream_sync.rs` runs p2panda-sync's `TopicLogSync` without live
  mode over any framed stream (`stream_codec.rs`: 4-byte length plus CBOR,
  length checked before buffering; frame ceiling 1 MiB default, 16 MiB hard
  maximum; one deadline; received operations through `processor_accept`). The
  messages are the ones p2panda-net sends (D8). `mere-transport`'s default-off
  `sync-lane` feature adds `SyncLane`, which reserves from the setup gate, runs
  native or Noise security, applies an owner `allow(peer)` rule and only then
  syncs. It lives in `transport`, not `murm-sync`, because `transport` already
  depends on `murm-sync` for the address book. Converges over memory, Noise on
  loopback TCP, Reticulum and p2panda; `NoiseListener` stays outside
  `Transport`. The `ReticulumTransport` header now cites RNS 1.5.7.
- **R3 behaviour.** A forged operation ends the session (p2panda-sync verifies
  signatures while decoding); a signed out-of-policy operation is declined and
  the session continues. An operation over the frame ceiling fails the session,
  so large content belongs on M12's resource carriage. Both ends must agree on
  the topic beforehand; `TopicLogSync` doesn't exchange it. Received operations
  pass a 4096-event backlog, and a slow `accept` reports `lagged`, which calls
  for another round. The gate authorizes the carrier peer; edition authority is
  the `OperationPolicy`'s. Over Reticulum the expected Noise key differs from
  the destination identity, so it comes from whoever named the peer.
- **R3 open.** The 24 MiB snapshot over Reticulum (M12's resource carriage),
  the R4 resolver and gossip over non-iroh carriers.

### Wave 2, 2026-11-05

- **R1 published.** All eleven `p2mere-*` 0.8.0 crates are on crates.io
  (`p2mere` and `-core`, `-auth`, `-encryption`, `-store`, `-discovery`,
  `-spaces`, `-stream`, `-sync`, `-net`, `-blobs`), from `merely-made/p2mere`
  tag `p2mere-0.8.0` (`b6899c09`). The dry run found that `p2panda-sync` and
  `p2panda-net` use `mpsc::Sender` as a `Sink` without enabling
  `futures-channel`'s `sink` feature; workspace unification hid it, upstream
  has the same gap, and the fork now enables it (`UPSTREAM.md`). crates.io
  allows five new crate names in a burst, then about one per ten minutes.
- **R1 consumers are a family repin, not a manifest edit.** Roots reach p2panda
  both directly and through git dependencies on each other (eidetic, murm,
  dramatis, gemot, knot-editor, mere). A repo that switches to `p2mere-*` while
  another in the same graph still resolves the patched `p2panda-*` gets two
  copies of each type. The switch has to move every repo in dependency order
  with their locks, as one coordinated refresh.
- **R7 key tiers.** Dramatis `cc7b99e`: Personae `UnassociatedKeypair` (fresh
  Ed25519, zeroized, no raw-seed constructor, persisted only by
  `seal`/`unseal`) and a sealed `CarrierKeypair` trait implemented only by
  `DerivedKeypair` and `UnassociatedKeypair`, so a master key still can't reach
  a transport; insigne `AssociationClaim` with `AssociationRevocation`, signed
  by the subject key and scoped by `CapabilityScope`. Murm `e92fe5f17ed`:
  `P2pandaTransport::builder_for` and a new `ReticulumTransport::builder_for`
  take `&impl CarrierKeypair`; an unassociated key's Reticulum identity shares
  neither half with the persona. The I2P plan records the D3 decision
  (unassociated by default on anonymous carriers).
- **R7 open.** SAM destination keys don't yet come from `UnassociatedKeypair`;
  an association claim proves only the subject's consent (no associate
  countersignature or revocation ledger); unassociated keys aren't wired into
  wallet or vault custody; no Tor transport. Arti 0.48.0 looks buildable
  without compiled C with `default-features = false`, `tokio`, rustls and no
  `static` or `compression`, but still links the system libsqlite3 through
  `tor-dirmgr`; that needs a `cargo tree` check. Murm's `--lib` kill/timing
  tests and `seed_residue` didn't finish clean under load and weren't compared
  with unchanged main.
- **R4 address grammar.** `gemot-address` (`gemot/crates/address`, no
  dependencies; re-exported as `gemot_site::address`):
  `knot://<64-hex key>/<path>[@<64-hex edition>]` and the governed
  `knot://<64-hex anchor>.moot/<path>[@…]`. Display round-trips for Copy link;
  hosts and pins render lowercase; today's Personae-root addresses parse
  unchanged. Turnstone's `parse_place_held_address` uses it (gemot `3b4c254`,
  `9cfa40f`, `37e7a77`, `e3cc052`; turnstone `b73883d`). `mere://` names no
  Moots or co-op sessions today.
- **R6 first slice.** gemot-site `clip.rs`: a `ClipPayloadV1` named by its
  blake3 digest in a `Clip` edition with provenance; `open_clip` verifies from
  the address key; `republish` starts a new log under the republisher's key
  whose first edition names the source in `fork_of`. Knot `EditionFrame`
  (knot-editor `77e2818`) opens an address and republishes, returning the new
  Copy-link address. Turnstone's `clip_edition` builds the edition, but the
  live clip path doesn't sign one yet: it needs R7's custody wiring.
- **R4/R6 open.** The `.moot` host carries one 32-byte value while
  `CollectionRef` is two (`moot_id`, `collection_id`), so the resolver has to
  settle which it names. `fork_of` names an edition, not its log key, so a
  fork carries the source address only as an unverified hint. Clip payload
  digests stand in for blob manifests until M12. Named-recipient grants aren't
  built; clips and forks are reference or bearer only. Reader history over clip
  frames is untested.
- **Pins.** Mere still pins eidetic `7a3453a`; Knot and Turnstone reach
  gemot-site at eidetic `a7a1458` beside their older copy, and moving only
  their eidetic rows breaks `pandect` (`FacetId` mismatch). `cargo update -p
  knot-editor` in Turnstone drags all of Mere, Genet and Dramatis main, so
  Turnstone takes gemot-site directly under `authoring` until a family refresh.

### Wave 3a, 2026-11-05

- **R8 intake.** `merely-made/p2mere` branch `intake-20261105` (`6e123c61`)
  merges upstream `d01898bf` (59 commits). The store traits changed
  incompatibly (every write is an explicit `_tx` method; `KeySecretsStore`
  became `PreKeyBundlesStore`), so the next release is 0.9.0 and Muniment must
  follow. Upstream's `tx!`-on-`SqliteStore` use in the address-book actor was
  declined under D8. Spaces brought awaitable `PendingSpace`/`PendingGroup`,
  removed-member queries and a compact `AuthenticatedMember` encoding (a spaces
  wire change); net moved to iroh 1.4.1; encryption to hpke-rs 0.8
  (RUSTSEC-2026-0387). The umbrella's `tests/api.rs` still hangs in five tests.
- **R8 evaluation (audit R25).** Probe `gemot/probes/spaces-eval` (gemot
  `510b68f`, outside every workspace), 16 tests. Spaces meets concurrent add and
  remove, nested devices, key rotation, offline revoked peers and cold reopen;
  partial on Muniment without SQLite (traits are backend-neutral, but the
  store's `spaces` feature forced SQLite, now fixed for 0.9.0); fails on a
  forward-secure profile (data scheme only; see D12). Gemot's `MootGroup`
  converges on members but its epoch number depends on delivery order; it holds
  no nested-group state. Spaces has no Personae-root attestation and no epoch
  forgetting. Recommendation: adopt spaces for generic group transitions and
  key agreement (most of Stickleback's `group_session.rs` and `group_lane.rs`),
  keep Personae attestation, epoch forgetting and the Muniment-backed durable
  lane, and keep Gemot's constitution and authority meaning. A Muniment
  implementation of the six store traits `Manager` needs is the first step; the
  probe's `MapStore` is a sketch.

- **Repin.** Every root moved to `p2mere-*` 0.8 under the old dependency keys
  (eidetic `1fcdb35`, murm `bc82e1aa92e`, gemot `c9e5ebf`, esp `3e97257`,
  knot-editor `6ea3ce2`, cleromancy `7e0f5ba`, mere `8b8caef`, turnstone
  `cc9cc6d2`, isocosm `86d07fab`, merelyllc.com `b8cb629`); no root resolves
  `p2panda-core`. Frozen receipt harnesses keep their `mark-ik/p2panda` tags by
  rule. Left open: Mere holds cleromancy-core at `8cc4f83` and the old genet and
  netrender revisions, because current genet pins vello by exact rev under new
  package names (a render-stack refresh, not this plan's); three
  `mere-resident` lib tests fail, one (`ActionNotOffered`) identically on
  untouched main and one on a too-long socket path in a worktree; stale
  `mere-p2panda-net` comments in isocosm and turnstone manifests.

### Wave 3b, 2026-11-05

- **R4 resolver.** Murm crate `murm-resolve` (murm `d73851580a6`) and gemot
  `cd795b0`. The governed form is `knot://<moot id>.moot/<collection id>/<page>`:
  `CollectionRef` is a pair and one Moot holds many collections, and a hash of
  the pair as host would hide the Moot from the reader (lane's choice, not yet
  Mark's). The address carries no founder; the reader supplies the one it trusts
  through `ReaderTrust::for_address`. A holder answers held/not-held for a key
  before sync, so one ALPN serves many logs; the reader admits only editions
  signed by the address key, refuses hints that lead elsewhere and checks `@`
  pins. Memory, Noise/TCP hints, p2panda (address-book topic tags; the reader
  does not `subscribe`, which would announce what it reads) and Reticulum
  (per-key ALPN, reader binds with `watch_alpns` and never announces) are tested
  in-process. I2P and onion hints parse but have no adapter. Open: a wildcard
  Reticulum watch needs announce-name filtering in retinue; a stale holder still
  verifies, so the result is the newest edition the contacted holders had;
  governed reads fetch Moot history separately before `verify_reader_evidence`.

- **D12.** Eidetic `e669eb2`: Stickleback `MessageSession` on `message_scheme`
  (one-time pre-keys per peer, signed under the group-scoped Personae key),
  `GroupKeyLane` generic over both session kinds, and
  `GroupEncryptionProfile::bounded_durable()` keeping 8 epochs through
  `forget_authorized`. Murm-sync `LaneKind` defaults to `Conversation`
  (forward-secure); `Durable` takes the bounded window. Gaps, all fork-side
  under D5: membership uses upstream's test-only `AckedTestDgm` (no re-adding a
  removed member, no production ordering); `DcgkaState` fields are private, so
  pre-key registration edits serialized state (making them `pub` in p2mere
  removes that); frames must arrive in causal order; no multi-device; no
  pre-key rotation; the retention window counts epochs, not time.
- **0.9 move.** Every root on `p2mere-*` 0.9 with one copy of p2mere, gemot,
  murm-sync and stickleback (eidetic `4aaa64d1`/`5182e4e`, murm `0aaf4b16d3d`,
  gemot `120bac79877b`/`9912dc85`, esp `27fd59188c50`, knot-editor `81185a2`,
  mere `a1f2c15ff`, isocosm `9568dfd8`, merelyllc.com `037863fc`, turnstone
  `8a980768`), except cleromancy: moving Graphshell onto current Mere needs
  Mere's newer genet, netrender and vello, so it waits on the render-stack
  refresh. Turnstone holds genet and netrender at its locked revisions through
  patch tables, and its lock still lists unused 0.8 optional deps of its old
  Mere and Knot pins. Muniment needed nothing; Stickleback's `MunimentStore`
  took the `_tx` traits with a no-op `Transaction`.

## Progress

- 2026-11-04: D9–D11 record recipient control, link-first delivery and the
  owner's authority boundary; R6 separates references, bearer/public releases
  and identity-restricted grants. R9 distinguishes publication from retrieval
  gateways and states route disclosure and verification limits. Design only.
- 2026-11-03: plan written with the brief; decision D1 recorded.
- 2026-11-03: D2 (both authorities), D3 (three address tiers) and D4 (HTTP
  compatibility without exposing the publisher) recorded; R9 added.
- 2026-11-03: redundancy pass folded in; D5 (tailor the fork) and D6 (solo Moot
  optional) recorded; R1 met; R3 drops its new chunker and the Noise
  `Transport`; R7 maps D3's tiers onto Personae; R8 executes audit R25.
- 2026-11-03: D7 (publish the fork as `p2mere-*`) and D8 (coherence across
  transports outranks upstream compatibility) recorded; R1 reopened as the
  fork migration and given the `fork` lane.
- 2026-11-05: references refreshed after the site extraction to Gemot and T7's
  `murm-sync`; R5 cites Gemot's relay pools plan; R9 notes `site-model`'s HTML
  generation; `spaces` now waits for the fork and an upstream intake. Wave 1
  started: `fork` (R1, up to publishing), `record` (R2), `carriage` (R3).
- 2026-11-05: wave 1. R2 first slice landed (eidetic `4a5b446`, `a7a1458`;
  gemot `3ed2ba4`, `847b4af`; murm pin `b6bedc30e21`). R3 stream-sync slice
  landed (murm `418d29a300e`, `b6ed4df6189`). R1: the fork moved to
  `merely-made/p2mere` (the old `mark-ik/p2panda` URL redirects), branch
  `p2mere` at `96d14386` renames eleven packages to `p2mere-*` 0.8.0 with
  library names unchanged and adds `UPSTREAM.md`; the umbrella publishes as
  `p2mere` (Mark's call). The umbrella's `tests/api.rs` hung for about ten
  minutes in four tests and was not checked against the untouched tag.
- 2026-11-05: wave 2. R1 published (`p2mere-*` 0.8.0, tag `p2mere-0.8.0`); R7
  first slice (dramatis `cc7b99e`, murm `e92fe5f17ed`); R4 grammar and R6 first
  slice (gemot `3b4c254`…`e3cc052`, knot-editor `77e2818`, turnstone `b73883d`).
- 2026-11-05: D12 recorded (forward-secure conversations, bounded durable
  data). R8 intake and evaluation recorded; 0.9.0 prepared with the store's
  SQLite coupling removed.
- 2026-11-05: `p2mere-*` 0.9.0 published (tag `p2mere-0.9.0`, `66904f86`):
  the upstream intake plus the store's SQLite decoupling. The family repin onto
  0.8 landed; moving it to 0.9 waits on Muniment's store-trait update.
- 2026-11-05: wave 3b. R4 resolver landed (gemot `cd795b0`, murm
  `d73851580a6`); the 0.9 move and D12 lanes are running.
- 2026-11-05: D12 wired (eidetic `e669eb2`; Murm through the 0.9 move) and the
  family moved to 0.9 except cleromancy. R5 holders and R7 follow-ups next.
