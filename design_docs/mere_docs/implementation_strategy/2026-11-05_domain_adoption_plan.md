# Domain adoption: every application's data in the reservoir

**Status:** 2026-11-05: opened as a draft for Mark's review. Nothing is
scheduled. D1 to D6 are ruled, including reservoir custody of Knot's
operation store. Everything marked *proposed* waits for a ruling. Only Cleromancy is in the reservoir
today.

**For:** moving each family application's own data into a mere of the
identity's reservoir, so that it follows the person to every device and any of
the identity's applications can open it. Opened by Mark on 2026-11-05, after
re-ruling Woodshed's custody: "Thinking about how it could apply to other apps,
our Cleromancies, Redshanks, Signalman, Knot Editor, the whole 9 yards, all of
those." He set the bar for replication the same day: "I would, by default, like
for even my working state to be replicated to my phone ... the chords and
whatnot that I'm working with, the progression I'm working with. I'd like for
that to definitely come with me."

**Not here:** the reservoir's own phases (V2r to V5, C1, PF), which stay in
[the reservoir plan](2026-10-28_reservoir_plan.md); device pairing, which is
`2026-10-28_device_pairing_plan.md`; each application's own move, which its
plan carries (Woodshed's is
`woodshed/design_docs/2026-11-05_woodshed_mere_plan.md`); communal records
that belong to a moot or a community rather than to a person.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8).

## Decisions

### D1. The store owns its lifetimes (Mark, 2026-11-05)

"The store itself" (distillery extraction D19, closing F1). Opening, durability
and closing of every mere belong to the reservoir store (pandect's reservoir
and the resident library). A Djinn-launched resident and an application
embedding the resident only execute it. A device that runs no Djinn, such as a
phone, embeds the resident.

Links: distillery-extraction D19 · summary store lifetimes belong to the
reservoir store, not to a process

### D2. Working state follows the person (Mark, 2026-11-05)

Authored data replicates to every device of the persona by default, working
state included, not only material the person has saved. Only device facts stay
local. Woodshed's custody ruling 6 applies this to Woodshed's whole domain.

Links: woodshed survey ruling 6 · summary replication is the default for
everything authored; device facts stay local

### D3. Conversations follow the identity across transports (Mark, 2026-11-05)

Answering O1: "it would be nice if conversations could follow you across
transports and devices ... not knowing the costs, being naive, i'll say all of
'em i can link to my identity or relevant derived identities." A
`conversations` domain holds every conversation the person can link to their
identity or a derived identity, whatever the transport (Murm's carriers,
Misfin, Reticulum through Outrider). It is the persistence home that
`crates/shell/comms` lacks. The person's copy moves; channel keys and other
secrets stay in custody, and radio, station and device journals stay local.
The costs are unmeasured.

Links: answers O1 · summary every identity-linked conversation, on any
transport, in one conversations mere

### D4. The notes mere is Knot's operation store (Mark, 2026-11-05)

Answering O2: "a notes domain sounds like a good target for clips. but why
can't that be knot's operation store?" It can. The `notes` domain is backed by
Knot's Stickleback operation store, and clips land there. Mark confirmed
on 2026-11-05 that the reservoir holds the operation store under D1.
Applications, including Knot, reach it through `mere/notes`. Knot supplies
the operation semantics for validation. The reservoir owns durable admission
and store lifetimes.

Links: answers O2 · summary notes and clips live in Knot's operation store,
held by the reservoir store

### D5. Domains by data: one music domain (Mark, 2026-11-05)

Answering O3 ("yeah") and Woodshed's domain name ("music, i guess"). Woodshed
and Hocket share one `music` domain, since both record phrases through
`hocket-model`. Mark also floated a music library ("we've no musicbee library
or anything (i guess redshank could do something like that???)"): if one is
built, it joins `music`. Redshank's podcast listening stays `listening` until
then.

Links: answers O3 · summary Woodshed and Hocket share the music domain; a
future music library joins it

### D6. Sealing in two layers (Mark, 2026-11-05)

Answering O4: "persona key is non negotiable for identity-associated access
... but the vault key is. so there should always be that personal encryption,
but you can have that additional domain-level protection? but maybe you don't
need to encrypt and protect your notes, and maybe you do." Every mere is
sealed under the persona's key, always. The persona also scopes the
reservoir: it lives under `personas/<persona>/reservoir/`, so the persona is
the index as well as the key. A domain may add its own vault key on top, and
each domain chooses whether to. Notes may or may not.

Links: answers O4 · summary persona sealing always; a domain-level key is
optional per domain

## The pattern every domain follows (proposed)

A domain is a kind of data, not an application (reservoir ruling of
2026-09-23: "two browsers should share a mere"). Each adoption needs:

1. **A pure domain crate** with no I/O that the resident can depend on. It
   holds the record types and one complete-candidate validator, as
   `cleromancy-core` supplies `validate_divination_graph`.
2. **Globally unique ids and real revisions.** Per-device counters and
   positional indices collide once two devices write.
3. **A split of what moves from what stays.** Authored content moves. Device
   facts stay local: window geometry, audio, MIDI and radio devices, caches,
   search indexes, file catalogs and anything re-fetchable.
4. **Large media as content-addressed blobs** beside the graph, fetched on
   demand, so a phone does not have to mirror every take or enclosure.
5. **Registration** in both the daemon and the embedded composition, through
   the shared registry (M1), never by editing hard-coded call sites.
6. **A migration ruled per application.** No application invents one. The
   proposed shape: a one-time import of the existing store, with the source
   kept read-only until the imported mere is verified.
7. **No downgrade at rest.** Every mere is sealed under the persona's key
   (D6, M3); a domain may add its own key.
8. **Communal data stays with its community.** A personal mere holds the
   person's own slice: their copies, roles and references.

## Mere's shared work (proposed)

### M1. A domain registry for both compositions

Today the daemon (`ports/mere-resident/src/bin/djinn.rs`, around line 1406)
and the embedded reservoir (`embedded_reservoir.rs`, around line 250) are
hard-coded to install Cleromancy's authority. Replace them with one registry
that both compositions read, where each domain names its id, its validator
and its owning application.
- Done when: Cleromancy's domains are served from the registry in daemon and
  embedded modes with the existing two-process tests passing, and a second
  domain is added without editing either composition.
- Needs: nothing new.

### M2. Validators required by registration

`serve` refuses a domain without a validator only for `divination` and
`personal-facts` (`resident_mere.rs`, around lines 357-364). Every other domain
is served under generic graph authority, which breaks the no-bypass
constraint. Make every registered domain require its validator, and refuse
unregistered product domains.
- Done when: an unregistered or validator-less product domain is refused in
  both compositions.
- Needs: M1.

### M3. Meres sealed at rest

The reservoir store (`crates/system/pandect/src/reservoir.rs`) and the
per-mere stores carry no sealing today. Woodshed's sessions and Knot's vault
are sealed now; moving them in unsealed would downgrade them. Per D6,
seal every mere under the persona's key released by custody (Dramatis's
`castellan-client` already seals retained payload bytes), with an optional
second, domain-level key that each domain may choose.
- Done when: a mere's durable files are unreadable without the persona's
  released key; a domain that chose its own key also needs that key; and a
  locked vault refuses reads and writes as Woodshed's `Custodied` backend does
  today.
- Needs: nothing beyond D6.

### M4. Replication across the persona's devices

D2 makes this a requirement, not a nicety. It is listed in the reservoir plan
as "Sync, branch and merge of meres across the identity's devices, which rides
Stickleback personal sync and codicil lineage (tracked, not scheduled)". The
resident's personal sync knows only `mere.graphshell/personal-graph/v1`
(`ports/mere-resident/src/personal_sync.rs`), and the expanded personal-sync
reopen test hang is still open.
- Done when: an edit to a registered domain on one paired device appears on a
  second, concurrent edits from both merge or surface as a branch, and the
  reopen hang is resolved.
- Needs: M1; `2026-10-28_device_pairing_plan.md` D1 and D2.

### M5. Blobs fetched on demand

A content-addressed blob store per mere, replicated by reference, with bodies
fetched when opened and an eviction budget per device.
- Done when: a take or enclosure recorded on one device opens on another
  after a fetch, and evicting it locally loses nothing.
- Needs: M4.

## Applications

| Application | Domain (proposed) | What moves | Stays local | Readiness | Validator source | Blockers |
|---|---|---|---|---|---|---|
| Cleromancy | `divination`, `personal-facts` | Already in: readings, charts, reflections | Scrubber and UI selection, sync consent | Replay-checked revisions | `cleromancy-core` | Its plan's O1, O2, O7 |
| Woodshed | `music`, shared with Hocket (D5) | Sets, working Sets, progressions, song, history, relationships, takes | Window, audio and MIDI devices, view state | Per-Set counters, positional song bars, `Revision(0)` overview, non-canonical history JSON | `woodshedding` plus checks carved out of `woodshed-core` | Its plan |
| Hocket | `music`, shared with Woodshed (D5) | Projects: session, history DAG, media | Audio device | Best: UUIDs, content-hashed media, mergeable history | `hocket-model` (`ProjectBundle::validate`) | Unsealed today; no node authorship |
| Redshank | `listening` | Library, subscriptions, notes and voice notes, progress, pins, transcripts | Enclosure cache, budget, volume, rate | Weak: timestamp note ids, whole-model JSON | `redshank-model` (pure, typed errors) | Unsealed; id scheme |
| Ringdown | none of its own | Presets and bindings ride Woodshed's settings | BLE pairing, MIDI ports | Positional bank slots | `ringdown` profile checks | Instrument has no read-back |
| Knot Editor | `notes`, backed by its operation store (D4) | Sealed document vault, retained captures, clips and relations | Search index, file catalog, `knot-sync.json` writers | Stable ids with revisions | `knot-document` and its operation semantics | Integrate the operation store into the reservoir (D4) |
| Turnstone | `browsing` | Session graphs, trails, recycle bin, retained clips | Cookies (persona custody), windows, workbench, page bytes over budget | UUID sessions; no domain crate | New work | Generic graph needs a validator |
| Signalman / Murm | `conversations` (D3) | Every conversation linked to the identity or a derived identity, any transport; contacts | Radio and device journals, station grants, channel keys | Journal-counter ids | `message.rs` read model, maybe | No persistence home yet for `shell/comms`; cross-transport identity linking |
| Isocosm | `worlds` for personal saves | Mesocosm and Eponym saves; a GM's own secrets | Session network state | Pure `isocosm-core`; revisions unclear | `isocosm-core` | Campaign retirement plan F3 |
| Gemot | the person's slice only | Roles, standing and membership references | Relay, pool, cache | Operation hashes; deterministic folds | Per-lane `verify` | Moot records are communal |

## Order (proposed)

1. M1 and M2, since every new domain needs them.
2. M3 before any sealed store moves in.
3. Woodshed as the first new consumer, as its owner has ruled.
4. Redshank next: its model crate is already pure.
5. Hocket, into the same `music` domain (D5).
6. Knot's operation store as the `notes` mere (D4).
7. Turnstone, Signalman's `conversations` (D3), then the personal slices of
   Isocosm and Gemot.

M4 and M5 run beside steps 3 to 7. No application's move is accepted until
its data reaches a second device.

## Open questions

- **O1 to O4** were answered on 2026-11-05 by D3 to D6.
- **O5. Settings that follow the person.** Theme, accessibility and pedal
  bindings follow; device choices do not. Is that the general rule?
- **O6. Linking derived identities to conversations (D3).** How a transport
  identity (a Reticulum or Misfin address, a station key) is attested as
  derived from the persona, so its conversations can be claimed.
- **O7. Which domains add their own key (D6).**

## Findings

- 2026-11-05: data inventories of all ten applications (from fresh clones)
  found only Cleromancy in the reservoir; no reservoir sealing; replication
  limited to Graphshell's personal graph; validator composition hard-coded to
  Cleromancy in both compositions.

## Progress

- 2026-11-05: plan opened. D3 to D6 recorded the same day.
