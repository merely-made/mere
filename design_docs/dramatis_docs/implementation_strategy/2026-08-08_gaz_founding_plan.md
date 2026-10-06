# Gaz Founding Plan

**Date**: 2026-08-08
**Status**: M0 landed 2026-08-08. On 2026-09-23 Mark ruled the anchor (a typed
key, a `did:plc`, or a local id), reaffirmed it over the standards survey's
first-pass DID grading (amended to match), and accepted the three M0.5
proposals from that day's critical pass: anchoring on a peer's root with
attested keys held concurrently, a whole-identity Reticulum key (verified
against Reticulum's reference implementation), and `Anchor::Local` for keyless
contacts. JSContact is ruled *an* exchange format (M1). M0.5 landed the same
day, and `TypedKey` then moved to insigne, as Mark ruled. M1 keeps sealing at the host and JSContact at the exchange boundary. M2 is
drafted with done-conditions. The
retained-proof slice of M2 landed on 2026-09-29 under Insigne phase D;
M1's storage gate is implemented on 2026-09-29 and its host sealing gate on
2026-09-30; JSContact exchange landed 2026-09-30, completing the M1 library
gates. The first M2 address-intake and WebFinger adapter slice landed
2026-09-30; checked key/PLC intake and application wiring remain open.
**Scope**: the contact layer, standalone. The record model, the persona-scoped
book, then storage over muniment, then the adapters that turn resolver output
into records, then mere reconciliation.

**Related**:

- Mere's [contact and remote-identity model brief](https://github.com/merely-made/mere)
  (`design_docs/mere_docs/research/2026-06-15_contact_identity_model_brief.md`)
  is the founding spec. It settles the *them* side of identity; this crate
  implements it.
- [`personae`](https://crates.io/crates/personae) owns the *me* side.
- [`muniment`](https://crates.io/crates/muniment) is the persistence seam M1
  rides.
- Mere's `ports/gazette` is the resolution sibling.

---

## 1. Why a crate and not a module

The record model is portable, has no mere dependency once it stops importing
`comms::Identity`, and has at least three prospective consumers: mere itself,
turnstone, and the radio companion. It also wants to be auditable on its own,
because it is the file that holds who a person knows. That clears the
crate bar. Publication is already done: `gaz` 0.0.1 was a reservation, and the
next publish is the first real one at 0.1.0.

## 2. Decisions

The brief's §9 left five shapes open. M0 settled four; Mark ruled the fifth
on 2026-09-23, and a critical pass the same day reopened parts of two of the
four (marked *Reopened* below).

**Contacts are anchored, not merely key-rooted.** The brief says a contact is
rooted on a stable key, but a key that can rotate cannot also be the map key
without re-filing the record on every rotation. So a record is filed under its
**anchor**, the first key ever seen, and `current_key()` is the newest. The key
list only grows. Consequences worth having: a rotation never moves a record, a
message signed with a retired key still resolves to its owner, and the
key-rooted rule becomes enforceable rather than aspirational.

**Reopened 2026-09-23: which key is the anchor.** "The first key ever seen"
assumes a person's keys form one line, each replacing the last. The stack's
own peers do not look like that (§5). A personae peer shows a different
*derived* key per protocol (the mesh author key is derived under
`mesh-author`), each carrying a `DerivedKeyAttestation` signed by one durable
master key, plus device keys delegated by master-signed certificates; a
Merely user's radios are further keys derived per station. These are
concurrent, not successive. Filed by first-seen key, one person becomes several
records, or their parallel keys are misread as rotations and `current_key()`
names an arbitrary one. Proposed in M0.5 and **accepted by Mark 2026-09-23**:
anchor on the peer's **root** where the peer proves one (for personae, the
master key an attestation names), and hold attested keys concurrently under it.

**The key-rooted invariant is enforced on both sides.** `Contact` cannot be
constructed without a key, the key list is private and append-only, and
deserialization rejects an empty list by name. State that has lost its root
fails to load instead of arriving as a contact rooted on nothing.

**Reopened 2026-09-23: people with no key.** The invariant means gaz cannot hold
anyone reachable only by email, phone, a misfin mailbox or a fediverse account
until a key turns up, which is most of the people anyone knows, and it leaves
JSContact import (ADOPT for gaz in the standards survey) with nowhere to put
most real cards. The contact brief's own reasons for key-rooting (§3: proof
chains presume a stable key; a record must survive host moves; it mirrors the
persona side) argue against rooting on a *handle*, not against a record that
has no key yet: a locally minted anchor survives host moves as well as a key
does. What the brief's rule really protects is *verification*, and that stays
per key and per endpoint. Proposed in M0.5 and **accepted by Mark
2026-09-23**: a `Local` anchor for keyless contacts, which widens gaz to
everyone you know rather than everyone with a key.

**Trust is one vocabulary, used per endpoint and per handle binding.**
`TrustState` is `Unverified | Pinned | Verified | Mismatched | Revoked`. The
brief named TOFU, signature, DID self-auth, and back-claim proof; those are
`ProofMethod` variants rather than separate states. `Mismatched` earns its
place because it is TOFU's whole point: a pin that stopped matching is either a
rotation nobody announced or someone in the middle, and gaz must not silently
choose the friendlier reading. `is_alarming()` marks the two states that have
to reach a person.

**Kith and kin are user-set, not trust-derived.** The brief asked. Deriving the
tier from verification would fold two axes into one and mean a peer becomes
close because their certificate checked out. Trust is a property of an address;
kith and kin describe a relationship. They stay separate.

**The anchor is a typed key or a `did:plc`, ruled by Mark 2026-09-23 and
reaffirmed the same day.** This restores the contact brief's own wording, "a
stable key or DID" (§3); M0's raw-keys-only root was the narrowing. The
[standards survey](../../2026-08-24_standards_survey_brief.md)'s first pass graded
DIDs PULL and carried only `did:key` and `did:web`, with no reason recorded. Mark
kept the ruling and the survey was amended to grade per method (its §5), since
`did:plc` is the only method that is per-account, rotating and self-certifying.
M0 took raw 32-byte keys as the root and treated `did:` strings as handles. Two
defects in that, both verified against live PLC data (the survey's §5 atproto
bullet holds the sources):

- *The key was untyped.* An Ed25519 key and a Nostr secp256k1 x-only key are
  both 32 bytes, and atproto signing keys are 33-byte compressed secp256k1 or
  P-256, which do not fit at all.
- *For atproto, the key is not the person.* An atproto signing key belongs to
  the hosting server, changes on every server move without the old key signing
  anything, and has been shared across accounts (all 895 accounts in an April
  2023 PLC sample shared two signing keys, 746 of them one key; `bsky.app` and
  `atproto.com` were created with the same key). Filed
  under its key, two people would collide on one anchor. The `did:plc` is the
  only identifier the protocol guarantees is per-account.

So the anchor becomes a `#[non_exhaustive]` enum:

- **`Anchor::Key(TypedKey)`** for peers whose identity is a key: personae
  (the master key, per M0.5), Nostr, murm, iroh, and stock Reticulum peers.
  `TypedKey` carries its algorithm (Ed25519, secp256k1, P-256, or a whole
  Reticulum identity) and is written in each family's own standard text form
  in human-readable formats (`did:key`, or `rnid`'s hex for Reticulum), raw
  tagged bytes in binary ones. Corrected 2026-09-23: a Reticulum identity was first placed here
  as "its Ed25519 half". It is one identity of two independent halves, which
  retinue fingerprints whole (`identity.hash()`), so the typed key holds the
  whole identity. Mark accepted it on condition that it follows Reticulum's
  standard; it does (§5).
- **`Anchor::Plc(PlcDid)`** for atproto. `did:plc` only: the identifier is a
  hash of the signed genesis operation, so it is self-certifying (re-derived
  from two live DIDs on 2026-09-23, with a tampered negative control). What
  it binds is that operation's rotation keys, usually the hosting server's, so
  a `Plc` anchor trusts whoever holds them; the PLC directory orders and serves
  later operations and cannot forge one. `did:web` stays a **handle**, because
  it is a domain, which is host-rooted identity in DID syntax. `did:key`
  collapses into `Key`.
- **`Anchor::Local(LocalId)`** for people with no key yet, accepted
  2026-09-23 (reopened decision above): a version-4 `urn:uuid` built from
  caller-supplied random bytes. The first key later proven for them is pinned
  under it; the anchor never moves.

Rotation keeps M0's rule for `Key` anchors: the anchor never moves, and the root
line only grows. For a `Plc` anchor the root line is a record of the signing keys
seen, not the root, and its continuity proof is a valid PLC operation rather
than a signature by the old key. gaz still verifies nothing; it records which
proof the caller checked (M2).

## 3. Shape

Six modules, all serde-only, no cryptography, no clock, no storage:

| Module | Holds |
|---|---|
| `key` | `ContactKey`, 32 bytes, hex in text formats and raw bytes in binary ones (becomes `TypedKey` + `Anchor` at M0.5) |
| `trust` | `TrustState`, `ProofMethod` |
| `handle` | `Handle`, `HandleKind`, normalization and matching |
| `endpoint` | `Endpoint`, `EndpointKind` |
| `contact` | `Contact`, `ContactTier`, the anchor rules |
| `book` | `ContactBook`, `PersonaScope`, `ScopeMismatch`, the queries |

Two dependency decisions, both deliberate:

- **No `personae` dependency.** gaz stores keys and compares them; it never
  verifies a signature. Depending on personae would pull ed25519-dalek, blake3,
  Argon2 and ChaCha20 into a crate that needs none of it. mere converts at the
  boundary, `Ed25519PublicKey::to_bytes()` to `ContactKey::from_bytes()`.
- **Not mere's `comms::ProtocolKind`.** That enum has two variants, Misfin and
  Murm, and is owned by the app. A contact book has to hold gemini capsules and
  ActivityPub actors it cannot message, so `EndpointKind` is its own open enum
  matching what the gazetteer already classifies WebFinger links into.

**Persona scoping is by construction**: one book per persona, and the book
carries its own scope label so a mis-filed load fails loudly through
`verify_scope` rather than quietly showing the work persona's people to a
burner. The label is an opaque string, so gaz never learns what a `PersonaId`
is.

**No clock.** Every timestamp is a caller-supplied `now_ms`. Deterministic
under test, fine on wasm, and it keeps the recency rules honest: every update
is monotonic, so a replayed or late event cannot rewind a record.

## 4. Phases

- **M0 — the model.** Done. Six modules, 41 tests plus a doctest, clippy clean
  under `-D warnings`, every file well under the 600-line ceiling.
- **M0.5 — the anchor.** Executes the §2 ruling. Nothing stores a book yet
  and nothing outside gaz consumes it (checked 2026-09-23), so there is no
  migration and no legacy decoder (DOC_POLICY §3). Done when:

  **Corrected 2026-10-06 (S14 pass):** the sentence above was false on its
  date. Retinue's Signalman desktop has imported gaz and loaded and saved a
  `ContactBook` through Muniment slots since retinue `864645e` (2026-08-19),
  at Mere `d82afa17` (Signalman's `messages.rs` in retinue, lines 5, 50 and
  128). M0.5 then changed the stored shape (a list of records instead of a map,
  the hex map keys gone), so Signalman's stored books are in the pre-M0.5
  shape. The insigne proofs plan's phase D records the same consumer still
  importing the pre-M0.5 `ContactKey`.

  **Open, raised by the S14 pass (2026-10-06):** how are Signalman's stored
  books, in the pre-M0.5 shape, handled when it repins? Options: a one-time
  migration in retinue; declare them disposable; a legacy decoder in gaz
  (DOC_POLICY §3 permits shims for real user data).

  - [x] `TypedKey` is an enum over Ed25519 (32 bytes), secp256k1 (33,
        compressed), P-256 (33, compressed) and a whole Reticulum identity
        (64). A Nostr x-only key converts
        losslessly as `0x02 ‖ x`, since BIP-340 fixes the even-Y point.
  - [x] Human-readable serde is the `did:key` string (base58btc multibase over
        the multicodec prefix: `ed01`, `e701`, `8024`); binary serde is the tagged
        raw bytes. Round-trip tests in both codecs, with `bsky.app`'s live
        secp256k1 signing key (from its PLC document) as a fixture.
  - [x] `PlcDid` accepts only `did:plc:` plus 24 characters of lowercase
        base32 (`a-z2-7`); anything else is refused by name.
  - [x] `Anchor` is `#[non_exhaustive] { Key(TypedKey), Plc(PlcDid),
        Local(LocalId) }`, the
        `ContactBook` map key, and a stored field on `Contact` rather than
        `keys[0]`.
  - [x] `by_key` returns every contact holding the key, not the first. Shared
        legacy atproto keys make one-key-one-contact false (§5).
        `mark_contacted` and `get`/`get_mut`/`remove` take an `Anchor`.
  - [x] **Accepted 2026-09-23 — anchor on the root, hold keys
        concurrently** (§2, reopened). A contact's keys become two sets: the
        *root line* (the anchor key and its rotations, successive, each
        proven by the key before it) and *attested keys* (derived and device
        keys, concurrent, each recorded with the root that attested it and its
        scope label, such as `mesh-author` or a station scope). `current_key()`
        gives way to `root()` and `keys_for(scope)`. An attested key never
        becomes the anchor: intake that sees a `DerivedKeyAttestation` files
        the contact under the master key it names. Keys seen with no
        attestation file as their own root until an attestation joins them to
        one, which is an M4 merge.
  - [x] **Accepted 2026-09-23, conditional on Reticulum's standard, which
        it meets (§5) — a Reticulum typed key.** `TypedKey::Reticulum` holds
        the whole 64-byte public identity, X25519 then Ed25519 as the
        reference implementation orders it, not the Ed25519 half. It anchors
        stock Reticulum peers and sits as an attested key under a Merely
        user's persona root. Its text form is the reference tool's own:
        `rnid` exports a public identity as undelimited lowercase hex of the
        64 bytes, so gaz writes those 128 characters. No multicodec exists
        for it, so it has no `did:key` form.
  - [x] **Accepted 2026-09-23 — `Anchor::Local` for keyless contacts** (§2,
        reopened). A version-4 `urn:uuid`, JSContact's recommended `uid` form,
        built from 16 random bytes the caller supplies, the same way gaz takes
        its clock from the caller. A `Local` contact may hold no key; the first
        key later proven for it is pinned under it (the anchor never moves). A
        keyless WebFinger import (M2) therefore starts a `Local` contact
        instead of being refused. `Key` and `Plc` contacts still hold at least
        one key: gazette resolves a DID before gaz files it.
  - [x] **Ruled 2026-09-23 — `TypedKey` lives in insigne**, whose grades
        start at "a bare key". Moved there (`git mv`, with its base58 and hex
        codecs and its tests); gaz depends on insigne and re-exports
        `TypedKey`, `KeyAlgorithm` and `KeyParseError`, so gaz's API is
        unchanged. Mark ruled insigne's core plain, serializable data with
        checking behind a feature, which is what lets gaz depend on it and stay
        crypto-free. `LocalId` moved onto the `uuid` crate (no default
        features), the stack's UUID type, instead of keeping a second hex codec
        in gaz.
  - [x] `cargo test -p gaz` and `cargo clippy -p gaz --all-targets -- -D
        warnings` green; lib docs and quick-start rewritten to the new types.
- **M1 — persistence.** An optional `muniment` feature; the core stays
  serde-only and default-featureless. Done when:
  - [x] `save_book` / `load_book` are generic over muniment's `Backend` and
        `Codec` (`SlotStore<B, C>`, async, as muniment is), at slot key
        `personas/<scope>/contacts`, where `<scope>` is the opaque
        `PersonaScope` string. Implemented 2026-09-29 behind `muniment`;
        the host selects Muniment's codec/backend features separately.
  - [x] `load_book(slots, &expected_scope)` runs `verify_scope` and refuses a
        mis-filed book by name; a missing slot returns an empty book for that
        scope, not an error.
  - [x] A malformed or rootless stored book fails to load; it never arrives as
        a partial book.
  - [x] Round-trip tests over `MemoryBackend` and redb, both a JSON and a
        binary codec, and a two-persona test proving one persona's load
        cannot return the other's contacts.
  - [x] **Ruled 2026-09-23 — at-rest sealing is the host's.** This file holds
        who a person knows. Access records are sealed at rest through
        castellan's `PersonaeHost::payload_sealer`; muniment itself has no
        sealing layer. Mark accepted the proposal: gaz stays plain and
        crypto-free, and the host supplies a sealed backend. Done when a
        host-side receipt shows the stored bytes carry no cleartext petname.
        Implemented 2026-09-30 through Pandect's `WalletSealedBackend` and
        Castellan's required `PersonaeHost::sealed_backend` supply point;
        `ports/castellan/tests/sealed_contacts.rs` checks JSON and postcard
        books in the entire closed redb file, then reopens and rechecks proofs.
  - [x] **Ruled 2026-09-23 — JSContact is *an* exchange format** (Mark: "the"
        was too strong). The standards survey grades
        JSContact (RFC 9553) ADOPT with "gaz (the contact store)" as consumer,
        though its §7 calls consumer assignments "a starting point, not a
        ruling", and its §5 prose sets JSContact against vCard, an exchange
        format. gaz keeps its own model at rest, and JSContact is an exchange
        format, built in this plan rather than deferred: import,
        export, and the card a persona publishes for others to import. Reason:
        what makes gaz gaz (anchors, the root line and attested keys, proofs,
        trust per endpoint and per handle, kith and kin) has no JSContact
        property, so a stored Card would be mostly vendor properties that no
        other reader understands, while gaz took on JSContact's shapes as
        internal constraints. In an exported Card, `uid` is the anchor as a
        standard identifier: `did:key:…`, `did:plc:…`, `urn:uuid:…` for
        `Local`, or Reticulum's bare rnid hex. Corrected against RFC 9553
        §2.1.9 on 2026-09-30: mandatory `uid` permits a URN, URI or free text;
        a version-4 UUID is preferred. Keys go in `cryptoKeys`, handles and
        service addresses in `onlineServices`, Gaz-specific state in host-domain
        vendor properties (§1.8.1). Implemented behind optional `jscontact`:
        `publish(&PublicCard)` accepts only explicitly selected public fields;
        peer `import` always makes unverified kith claims and ignores local
        state. Separate `export_contact` / `restore_contact` preserve the whole
        PRIVATE Contact, including u64 history encoded as JSON text, and refuse
        disagreement with public projections. Only the host's trusted backups
        may use restoration. Typed artifacts remain unchecked claims.
        Reticulum crypto resources are data URIs containing all 64 public bytes.
        The host supplies its controlled extension domain; other domains remain
        opaque. Imported source Cards retain unknown properties and original
        map IDs separately from Contact. Generated IDs survive reordering and
        unrelated additions. Foreign non-anchor uids require a caller-selected
        local id; PLC imports require a root-bearing identity claim. This is
        bounded exchange support, not full RFC validation, vCard conversion,
        resolver merging, Ledger UI or network publication.
        Receipt: 82 unit tests plus three doctests, scoped Clippy with warnings
        denied, and a Wasm production check. A deliberately inserted peer
        restore path makes the trust-isolation test fail; restoring the source
        returns the full suite to green. The JSContact-only production graph excludes
        Personae and cryptography, and the default graph excludes the optional
        JSON/URI exchange dependencies. Raw logs and the mutation receipt are
        retained in `C:\t\cargo-targets\mere\gaz-jscontact-receipts`.
        Next: M2 resolver intake.
- **M2 — resolver intake.** The seam where resolution meets storage. Gazette
  depends on gaz, not the reverse (gazette was promoted to a port on
  2026-08-23 and "composes gaz rather than replaces it"), so gaz cannot consume
  gazette's types. Implemented split: gaz owns the intake *rules* and their
  input types; gazette converts its resolver output into them. That dependency
  landed for supplied WebFinger address intake on 2026-09-30. Gazette depends
  on Gaz and converts its classified result into Gaz-owned claim types.
  Cryptographic intake, resolver networking hardening and live Ledger/store
  hosting remain open. Done when:
  - [ ] Every key after the first is recorded with the `ProofMethod` that
        justified it: for `Key` anchors, `Signature` by the previous root for
        a rotation and by the root for an attested key; for `Plc` anchors,
        `DidAuth` (a valid PLC operation). A key change the caller reports
        without a proof is stored and marks the contact `Mismatched`, so it
        surfaces in `alarms()`.
  - [x] **Ruled 2026-09-23 — gaz keeps the proofs themselves**, not only the
        `ProofMethod` that names them. A key recorded with a proof also keeps
        the artifact that proved it (a `DerivedKeyAttestation`, a
        `SignedDelegationCertificate`, a PLC operation) as plain data from
        insigne's core, so it can be checked again later, for instance against
        a newer revocation list, which is why notochord retains its session
        claims. The data types moved into insigne in the
        [insigne proofs plan](../../archive_docs/2026-10-06_completed_plans/2026-09-23_insigne_proofs_plan.md)'s phase A
        (`5364dfa0`, 2026-09-24); gaz keeping them is that plan's phase D,
        landed 2026-09-29. `KeyProof` retains attestations with their salt,
        signed certificates, or opaque protocol evidence with its format and
        method. Typed key relationships are checked on insertion and load;
        cryptographic checks and current authority remain the caller's.
        JSON/postcard reload and recheck are proven. PLC decoding/checking
        and the other intake rules below remain open; M1 persistence is complete.
  - [x] Intake adds endpoints as `TrustState::Unverified` and never downgrades
        or duplicates an endpoint already held at a stronger state; replaying
        the same intake is a no-op. Implemented 2026-09-30 through
        `ContactBook::intake_addresses`: typed handle/endpoint claims contain
        no trust, tier, note, recency or key fields; new claims are Unverified,
        while all existing state and alarms survive. A staged clone prevents
        partial mutation when an input or alias conflicts. Exact typed endpoint
        addresses and typed handle comparison make duplicate/reordered replay
        a no-op. Core tests cover every TrustState; sealed host receipts retain
        and recheck actual root/device proofs.
  - [ ] A handle binding can be raised by back-claim (the handle's own
        well-known document names the key or DID) through the same proof
        vocabulary.
  - [ ] A test replays a real PLC history (`bsky.app`'s four operations,
        ending in the 2023-11-09 server move in §5) as intake and ends with one contact, two keys, the
        second carrying `DidAuth`, and no alarm; the same history with the proof
        withheld ends in `alarms()`.
  - [ ] WebFinger output never becomes an anchor. A keyless import attaches
        its endpoints to the contact already holding that `acct:` handle, or
        else starts a `Local` contact (M0.5) that a key can later join. The
        [contact brief](../../mere_docs/research/2026-06-15_contact_identity_model_brief.md)'s
        committed position (§3: WebFinger is one endpoint resolver hanging off
        the contact, not the identity) still holds: the handle labels the
        record and never roots it. Keys come from the key-rooted resolver the
        brief says must return (§4, §9), or out of band (a murm invite, an
        iroh ticket, a QR code). A test proves both paths. The first WebFinger
        branch is implemented 2026-09-30: only the queried Acct handle selects;
        a unique match preserves its existing key/local/PLC anchor, no match
        requires a host-selected LocalId, and aliases never supply anchors or
        keys. Multiple matches, occupied fallback ids and secondary handles held
        elsewhere refuse the whole intake. Rooted and keyless fixture paths
        pass, including persona isolation after sealed reopening. Fresh key
        joining and key-rooted resolver intake remain open, so this broader
        checkbox stays open.
  - [ ] Checking PLC operations belongs to gazette, which owns resolution and
        can take the crypto; its atproto-did resolver is gazette work tracked
        in the port's README, not a gaz phase.
- **M3 — mere reconciliation.** Conversion glue for `personae` keys and
  `comms::Identity` endpoints, and the `Contact` rollup the comms model lacks.
  gaz has lived in mere's workspace since the 2026-08-10 dramatis subtree
  merge, so this is an in-tree path dependency, no longer the git dependency
  first planned.
- **M4 — recency and merge.** Duplicate detection when two records turn out to
  be one person, which needs a merge rule for conflicting petnames and tiers.

## 5. Findings

**The slot really was empty.** No `struct Contact` existed anywhere in mere
before this. The brief specified the record in June and nothing was built, so
M0 is the first implementation rather than a port.

**JSON forced the key encoding, and improved it.** A `BTreeMap<ContactKey, _>`
cannot serialize through JSON while the key is a byte array, because JSON map
keys must be strings. The round-trip test caught it. The fix, hex in
human-readable formats and raw bytes in binary ones, also makes a stored book
readable by a person, which is worth having for this particular file. Postcard
pays nothing for it.

**The brief's proposed struct could not be used verbatim.** Its
`endpoints: Vec<(Identity, TrustState)>` names mere's `comms::Identity`, which
would have made gaz mere-coupled and capped it at two protocols. Same intent,
independent type.

**2026-09-23: atproto keys are the server's, not the person's.** Checked
against the live PLC directory rather than assumed; sources, counts and the
re-derivation of `did:plc` live once, in the
[standards survey](../../2026-08-24_standards_survey_brief.md)'s §5 atproto
bullet. What they mean for gaz:

- **Shared signing keys**, so an atproto contact filed by key can collide with
  a stranger. Hence the `Plc` anchor, and `by_key` returning every match.
- **Key changes are never vouched for by the old key.** The authority is a PLC
  operation signed by a rotation key, so an "old key must vouch" alarm would
  fire on every routine server move. Hence `DidAuth` as its own proof in M2.
- **33-byte secp256k1 and P-256 keys**, which M0's 32 bytes cannot hold.
  Hence `TypedKey`. The live `bsky.app` key is M0.5's fixture.
- **The DID binds the rotation keys**, usually the hosting server's, so a
  `Plc` anchor trusts that custodian. Recorded in §2 so nobody mistakes
  self-certifying for self-sovereign.

Not checked: whether any account still active today signs with a shared legacy
key, and other hosting servers' key policies beyond the 2026 sample.

**2026-09-23: M0's key model does not fit the stack's own peers.** A critical
pass over M0's decisions, against the code:

- personae gives a persona one durable master key and binds every other key to
  it: `DerivedKeyAttestation` is "a master-signed statement binding one
  deterministically derived key to its identity root and derivation salt"
  (`crates/dramatis/personae/src/provider.rs`), and devices hold
  `SignedDelegationCertificate`s issued by the master
  (`crates/dramatis/personae/src/delegation.rs`).
- What a peer shows on the wire is a derived key, not the master: mesh authors
  sign with `derive_keypair(MESH_AUTHOR_SALT)` and prove it with
  `attest_derived_key` (`crates/mesh/mesh/src/directory.rs`), and a key minted
  for another protocol deliberately does not count as the mesh identity.
- castellan derives a Reticulum identity per station from a persona provider
  (`ports/castellan/src/reticulum.rs`), so a Merely user's radios are further
  concurrent keys under the same root.

M0's rules (anchor = first key seen; one growing list; `current_key()` = the
newest) model one key replacing another. They cannot represent one root with
several live keys, which is the normal case for personae peers. The fix is
proposed in M0.5.

**2026-09-23: a Reticulum identity is two keys, fingerprinted whole.** castellan
documents "two independent 32-byte secret halves: one for X25519 exchange and one
for Ed25519 signing", and retinue fingerprints a node by `identity.hash()`
(`retinue/crates/outrider/examples/direct_phy_ui.rs`). Anchoring on the Ed25519
half, as §2 first said, would let an X25519 half nobody proved ride on a trusted
anchor, and a sender would encrypt to it. Corrected in §2. The whole-identity
key was then checked against Reticulum's reference implementation, its only
specification: the 64-byte layout, the whole-key identity hash and `rnid`'s hex
text form all match (sources in the
[standards survey](../../2026-08-24_standards_survey_brief.md)'s §5
signalman/retinue bullet), which met Mark's condition for accepting it.

**2026-09-23: M0.5, what building it settled.**

- **serde stops at 32-byte arrays**, so `TypedKey`'s binary form is one byte
  string, gaz's tag byte then the key (`serialize_bytes`), not a derived
  enum. Tested through postcard, the codec muniment stores with, so M1's
  format is the one under test.
- **The stored book is a list of records, not a map** (`crates/dramatis/gaz/src/book.rs`).
  M0's map key repeated the anchor already inside each record, and the two
  could disagree on a hand-edited or corrupted file; now there is one copy,
  and a load refuses two records on one anchor. M0's JSON shape (hex map keys)
  is gone with it, which costs nothing: no book was ever stored (DOC_POLICY §3).

  **Corrected 2026-10-06 (S14 pass):** books were stored: Retinue's Signalman
  desktop had saved `ContactBook`s through Muniment since 2026-08-19, in M0's
  map shape (see the correction under M0.5 in §4).

- **Every fixture has a real source**, not invented bytes: the did:key spec's
  worked example, `bsky.app`'s live secp256k1 key, a live P-256 key from the
  PLC export, prns's RNS 1.4.2 identity, and the IETF base58 draft's examples.
  The identity's hash was recomputed as a positive control: the whole 64
  bytes reproduce prns's expected hash and the Ed25519 half alone does not.
- **The invariant tests bite.** Two were broken on purpose (an attestation
  by an unknown root accepted on load; an uncompressed secp256k1 prefix
  accepted) and their tests went red, then the code was restored.
- `contact.rs` crossed the 600-line ceiling with its tests, which moved to
  `contact_tests.rs` by `#[path]`, as personae's `sealed_record_storage_tests.rs`
  does.

**2026-09-23: WebFinger yields no key.** `gazette::WebFingerImport`
(`ports/gazette/src/lib.rs`) classifies endpoints and aliases and has no key
field. That is what the contact brief intends (§3): WebFinger fills in
endpoints on a contact rooted elsewhere. The gap is upstream: the key-rooted
resolver the brief says must return does not exist yet, so today every key
arrives out of band. First drafted here as an open decision, and withdrawn the
same day once the brief was read in full. The `Local` ruling later that day
changed the outcome but not the brief's point: a keyless WebFinger find now
starts a `Local` contact, and the handle still never becomes the anchor.

**2026-09-23: the dependency runs gazette → gaz.** The founding text placed
intake in gaz "rather than in the resolver" when the resolver was a sibling
crate. Since the 2026-08-23 promotion, gazette is a port that composes gaz, so
gaz can own intake rules but not consume gazette's types. M2 is re-drafted
around that.

## 6. Progress

**2026-08-08.** Founded. Reservation stub replaced by the M0 model. `cargo test`
41 passed plus 1 doctest; `cargo clippy --all-targets -- -D warnings` clean.
File sizes: book 294, contact 271, key 204, trust 122, endpoint 100, handle 100,
lib 76.

Next session starts at M1, and should confirm the DID question in §2 before
M3 makes the anchor type expensive to change.

**2026-09-23.** The DID question ruled by Mark: `Anchor::Key(TypedKey) |
Anchor::Plc(PlcDid)`, `did:web` a handle (§2). First proposed as typed keys
with DIDs as handles; reversed the same day once the PLC data in §5 was
fetched rather than deferred. M0.5 inserted ahead of M1 to execute it; M1 and M2
drafted with done-conditions; M2 re-scoped for the gazette → gaz dependency
direction; M3's "git dep" corrected to in-tree. No code changed. Open for
Mark: at-rest sealing (M1). WebFinger-only contacts, first drafted as open,
were already settled by the contact brief (§3).

**2026-09-23, later.** The §2 ruling is back with Mark: it was made without the
[standards survey](../../2026-08-24_standards_survey_brief.md), which grades DIDs
PULL with `did:key` and `did:web` "the only two methods worth carrying", and
atproto WATCH. The survey also grades JSContact (RFC 9553) ADOPT with gaz as the
consumer, which M1 as drafted ignores. Neither M0.5 nor M1 starts until both
are settled.

**2026-09-23, critical pass.** Mark reaffirmed the ruling over the survey,
rejecting "add it when a consumer arrives" as a posture, and asked that
earlier decisions be examined on their merits rather than obeyed. The survey
was amended (per-method DID grades; atproto's identity layer split from its
WATCH). `did:plc` self-certification was re-derived from live data rather than
quoted. Examining M0's own decisions against the code reopened two of them:
the anchor rule and key list (§2 and §5: personae peers present concurrent
attested keys under one master) and the key-rooted invariant (§2: it shuts
out keyless people). Three proposals now sit in M0.5 for Mark (root anchoring
with concurrent keys, a whole-identity Reticulum key, `Anchor::Local`), plus
JSContact's role in M1. The M2 WebFinger rule is conditional on `Local`. §2's
Reticulum line was wrong and is corrected.

**2026-09-23, rulings.** Mark accepted root anchoring with concurrent attested
keys and `Anchor::Local` outright, and the whole-identity Reticulum key on
condition that it follows Reticulum's standard. It does: checked against the
reference implementation's `RNS/Identity.py` and `rnid`, and against prns
(§5, sources in the survey), and the text form was switched from a house form to
`rnid`'s own hex. JSContact is ruled *an* exchange format, not *the*. The M2
WebFinger rule is no longer conditional: a keyless import starts a `Local`
contact. Still open: whether `TypedKey` lives in insigne (M0.5). M0.5 starts.

**2026-09-23, M0.5 landed.** `TypedKey`, `Anchor` (`Key`, `Plc`, `Local`),
the root line and attested keys, `by_key` returning every holder, and a book
stored as a list. `cargo test -p gaz`: 70 passed plus 1 doctest;
`cargo clippy -p gaz --all-targets -- -D warnings` clean; rustfmt applied.
File sizes: contact 396 (+ tests 280), key 483, book 462, anchor 366, encoding
156, trust 136, handle 129, endpoint 125, lib 102. One dev-dependency added
(postcard), one `Cargo.lock` line. Still open: where `TypedKey` lives. M1
(persistence) is next; its at-rest sealing question is still open (M1).

**2026-09-23, `TypedKey` to insigne; M1 sealing ruled.** Mark ruled `TypedKey`
into insigne and insigne's core plain data with checking behind a feature,
after a check of the stack found no contradiction: notochord already keeps
travelling claims separate from its local, non-`Serialize` `AdmittedPrincipal`,
and insigne's own docs call the insigne "the interchange artifact" and gaz "the
ledger of insignia received". `key.rs` and `encoding.rs` moved by `git mv`; gaz
re-exports the three key types, and `LocalId` now wraps `uuid::Uuid`. Tests
are unchanged in number: gaz 54 plus a doctest, insigne 16; clippy clean on
both. He also ruled M1's at-rest sealing the host's, keeping gaz crypto-free.
Open: whether personae's delegation certificates and attestations move into
insigne, which reverses the 2026-08-11 reconciliation's "personae owns the
grammar" and is tracked in the crate consolidation plan.

**2026-09-23, delegation split and kept proofs ruled.** Mark agreed the split:
personae's delegation data types move into insigne's core, issuing stays in
personae, and checking goes behind an insigne feature that returns local,
non-`Serialize` conclusions. He also ruled that gaz keeps the proofs themselves
(M2), which is why this session takes the move. The standards survey's PULL
grade was redefined the same day ("design for it now, implement in order").

**2026-09-29, M1 storage gate.** `gaz::save_book` and `gaz::load_book` persist
the complete own-model book through a caller's `SlotStore<B, C>`. Saving uses
the book's scope. Loading validates the whole model and then the expected
scope, returning an empty book only for a missing slot. `PersistenceError`
distinguishes store/codec failure from a `ScopeMismatch` naming both personas.
Neither malformed records nor an I/O failure are absorbed as empty history.
The `muniment` feature adds the floor without selecting a codec or backend;
the default data model has no persistence or crypto dependency.

Five new tests include four memory/redb × JSON/postcard matrix cases. Each
proves missing-slot behavior, separate persona slots, overwrite isolation,
mis-filed book refusal, rootless/mismatched-proof/duplicate-anchor refusal,
retention of refused bytes, and clearing one book without clearing the other.
The redb cases drop the database and reopen it before loading and rechecking
the real retained attestation. The unchecked test record's wire layout is
compared to the valid book before introducing malformations. Postcard erases
serde's custom error text; its portable refusal is a codec error. Scope errors
still carry both persona names under either codec. A fifth test proves opaque
scope labels are preserved rather than normalized.

Temporarily removing `load_book`'s scope check makes all four matrix cases fail
(exit 101); it is restored before the final complete suite. Gates are recorded
under `C:\t\cargo-targets\mere\gaz-m1-receipts` with source hashes and raw logs:

- `cargo test -p gaz --all-features --locked --offline -j 4`: **66 tests plus
  two doctests pass**.
- `cargo clippy -p gaz --all-targets --all-features --no-deps --locked --offline
  -j 4 -- -D warnings`: **pass**, scoped to Gaz as in Insigne phase D.
- `cargo check -p gaz --lib --no-default-features --locked --offline -j 4
  --message-format=json`: **pass**, showing the default core excludes Muniment and
  cryptographic dependencies.
- `cargo check -p gaz --lib --all-features --target wasm32-unknown-unknown
  --locked --offline -j 4 --message-format=json`: **pass**, showing portable persistence
  with no default Muniment codec or Insigne verification feature. Muniment's
  BLAKE3 storage hashing belongs to the optional floor.

Isolation was required after another build took the shared target and a
concurrent root-manifest repin selected uncached Genet `c5470fcb`. Gates use
committed Mere `ca2351b3` plus this owned Gaz patch in
`Code\worktrees\mere-gaz-persistence`, with
`C:\t\cargo-targets\mere\gaz` as the build output. The cache wait was a live
`cargo fetch --locked`, which was preserved. The gate worktree and output are
removed after integration; receipts remain. The concurrent Genet repin and
its Cambium changes are outside this receipt.

The first four M1 done-conditions are met. **M1 remains open** for the host-side
sealed-backend receipt and JSContact import/export/published-card exchange.
Gazette intake and its trust/alarm behavior remain M2, with no live Gaz host
integration claimed by this storage gate.

**2026-09-30, M1 host sealing gate.** Pandect owns `WalletSealedBackend<B>`
over the backend the host selects. Castellan's keeper supplies it through
`PersonaeHost::sealed_backend(persona, backend)`, refusing when a carry root
or staged private epoch is unavailable. Gaz's dependency graph is unchanged;
the host applies sealing below its existing `SlotStore` and `save_book` /
`load_book` calls.

The adapter reuses the real `WalletEpochSealer`. A versioned envelope retains
the epoch marker and ciphertext; the authenticated inner value binds the
logical slot key and carries a fresh random salt, so its exposed content digest
does not reveal equality or permit guessing a small record. Reads authenticate,
check the plaintext hash, and check the slot key before returning bytes.
Plaintext, corrupted envelopes, wrong personas/keys/epochs, and copies at
another key are errors. Refused bytes remain intact. All puts in a batch are
sealed before the underlying backend receives any writes.

The host receipt stages actual wallet state, obtains the adapter from
Castellan, saves two persona books, closes every redb and keeper handle,
checks the entire durable file for cleartext petnames, and reopens both books.
The encoded JSON and postcard inputs each contain the tested petname, so the
absence check is not merely an encoding artifact. A retained attestation checks
again after reopening. Wrong-persona reads, damaged ciphertext, plaintext
substitution, and slot transplantation refuse without erasing stored bytes;
clearing one persona preserves the other. Adapter tests separately exercise
ordered scans, listing, write batches, repeated writes, and explicit historical
epoch loading after rotation.

Validation on Rust 1.98.1, with raw logs and source hashes retained under
`C:\t\cargo-targets\mere\gaz-sealing-receipts`:

- `cargo test -p pandect --lib wallet_sealed_backend --locked --offline -j 4`:
  **three adapter tests pass**.
- `cargo test -p castellan --features keeper --locked --offline -j 4`:
  **76 unit tests, three host receipts, and one doctest pass**.
- Temporarily bypassing sealing in `Backend::put` makes both durable-petname
  tests fail at the cleartext assertion (exit 101). Sealed writes are restored,
  and the complete Castellan suite passes again.
- `cargo clippy -p pandect -p castellan --lib --tests --features castellan/keeper
  --no-deps --locked --offline -j 4 --message-format=json`: **completes with
  no diagnostics in the four added Rust files**. Existing diagnostics in
  Pandect, Castellan, and Kernel remain; this is not a whole-crate `-D warnings`
  receipt.
- `cargo check -p gaz --lib --no-default-features --locked --offline -j 4
  --message-format=json`: **pass**, with Gaz and Insigne featureless and no
  Muniment, Personae, or cryptographic library in the production artifacts.
- `cargo check -p pandect --lib --target wasm32-unknown-unknown --locked
  --offline -j 4 --message-format=json`, with the existing getrandom 0.3
  browser selector `--cfg getrandom_backend="wasm_js"`: **pass**. The adapter
  preserves Muniment's native `Send` / browser `?Send` future seam, requiring
  `Sync` only on native. This is a compilation receipt, not browser storage
  execution.

This slice reuses the primary `C:\t\cargo-targets\mere` output. It creates no
isolated worktree, target, or Cargo home.

This meets M1's at-rest host receipt, not a claim that an application has wired
Gaz into its contact UI. Keys remain visible, read/write transactions explicitly
return `NotTransactional`, and replay of an older authenticated value at the
same key is outside this adapter. Historical epoch supply and freshness remain
host responsibilities. At this sealing checkpoint, JSContact exchange was the
remaining M1 gate; it subsequently landed as `cf901f3d` on 2026-09-30. M1's
library gates are complete; M2's bounded address-intake slice is recorded next.


### 2026-09-30: M2 unverified address intake and supplied WebFinger adapter

The next coherent slice is the address-only seam between resolution and the
persona book. `gaz::intake` owns `AddressIntake`, `HandleClaim`, `EndpointClaim`,
the host-selected `NewLocalContact`, and atomic refusal/outcome types.
`ContactBook::intake_addresses` selects only by the primary typed handle,
stages additions, then inserts once. It never calls replacement import with a
resolver's Contact, so private names, notes, tiers, history and trust cannot
arrive as authority. Existing alarms remain visible. The fallback id cannot
replace an existing Local record, and aliases held elsewhere cannot join people.

**Comparison probe:** the old `Handle::normalized` helper lowercases every
family and exists for human lookup. Intake uses a separate identity comparator:
Acct account-name case is retained; scheme/DNS host case folds; unreserved ASCII
percent escapes decode and retained hex is uppercase. Opaque handles and
endpoint paths compare exactly. IPv6 literals use canonical parsed text; DNS
trailing dots remain distinct. This bounded parser accepts ASCII DNS/A-labels
and percent-encoded user bytes, without constructing IDNs or doing full PRECIS.
Leading percent escapes follow the correction proposed in
[reported RFC 7565 erratum 7998](https://www.rfc-editor.org/errata/eid7998),
not a claim that the erratum has been verified.

Gazette's `WebFingerIntake::from_import(resource, source)` accepts its existing
classified resolver result, retains the source separately, and projects valid
protocol addresses plus Acct/DID aliases as unverified claims. `did:key` and
`did:plc` aliases remain handles; they never promote identity keys or anchors.
Malformed and unmapped fields remain source-only. The wrapper preserves the
classified WebFingerImport, not unknown fields discarded by the original JRD
parser. Query/subject binding deliberately requires the same normalized account;
RFC 7033 §4.4.1 permits changed subjects, so account migration is refused here
as an explicit first-slice policy. The existing fetch path is unchanged.

`ports/castellan/tests/sealed_webfinger_intake.rs` composes this supplied-JRD
adapter with actual wallet epochs, `PersonaeHost::sealed_backend`, Muniment and
JSON/postcard redb. It starts from a reopened book, imports additional addresses,
replays before and after another reopen, preserves Pinned/Verified trust and
private relationship state, rechecks retained rotation/device artifacts, and
refuses cross-persona reads. A separate empty persona needs a caller-selected
keyless Local fallback. Entire closed redb files conceal petnames; both codecs
have a cleartext encoding positive control. This is a native host-composition
receipt, not live external resolution, Ledger UI or headed product evidence.

**Remaining M2 gates:** checked key mutations and durable key-change alarms,
back-claim upgrades, checked PLC history and Local key joining. Application
Ledger/recipient-picker hosting, async/persona-aware resolver fetching, HTTPS
redirect enforcement and account migration remain Gazette/host work. The old
blocking fetch path has no HTTPS-only redirect policy; this supplied-result
adapter neither changes it nor claims transport authentication. Personae,
Insigne and Dramatis models are unchanged; Castellan gains only a test consumer.

Final gates, all locked/offline. After integrating origin/main `8425cd73`
(Genet `b1eb3af1`), all seven gates passed again; the integrated commands and
raw logs are indexed by `gaz-m2-receipts/gates-integrated.json`.

- Gaz: **92 unit tests and four doctests**, including every trust state and
  refusal atomicity. Gazette: **20 tests**, ten new supplied-intake cases.
- Castellan `--features keeper`: **76 unit tests**, the three existing sealed
  contact receipts, the four new JSON/postcard × Pinned/Verified intake receipts,
  and one doctest. The Linux-only Secret Service target runs zero tests here;
  this receipt is native Windows.
- `cargo clippy -p gaz -p gazette --all-targets --all-features --no-deps
  --locked --offline -j4 -- -D warnings`: **pass**. The scoped Castellan receipt
  Clippy run reports no warning in the new test file; **161 existing
  Kernel/Pandect/Castellan warnings** remain in the JSON diagnostic log.
  This is not a whole-Castellan warnings-denied claim.
- Gaz production `--lib --all-features --target wasm32-unknown-unknown`:
  **pass**. Insigne's artifact has no features and this graph has no Personae,
  signature checker or resolver. Muniment's existing BLAKE3 storage hash remains
  in the optional persistence graph. Default native production is also **pass**,
  without JSON, URI, Muniment, network or cryptographic dependencies.
- Deliberately resetting held endpoint trust to Unverified makes both the core
  preservation test and JSON/Pinned sealed-host receipt **fail** (exit 101).
  The source is restored byte-for-byte; all full suites pass afterward. Mutation
  commands, source hash and raw logs are retained in `gaz-m2-receipts`.

Three delegated lanes supplied the Gazette adapter, sealed-host receipt and
independent contract review; the root owned Gaz, combined verification, docs,
lockfile and publication. All new Rust files remain below 600 lines.

Reuse the primary `C:\t\cargo-targets\mere`; raw logs live in `gaz-m2-receipts`. This
slice creates no isolated worktree, Cargo target or Cargo home.

### 2026-10-06: S14 pass

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere 535bca11, from the D2 record in support/doc-audit/d2/batch_39_s14_phase_b1.md: M0.5's and §5's "no book was ever stored" corrected with Signalman's Muniment-stored books, and their pre-M0.5 shape raised as an open question at M0.5.
