# Dramatis Tier Architecture

**Date:** 2026-09-30
**Status:** current. The tier's architecture of record: one account of identity
across the six crates and two ports that carry it. It synthesises the plans
listed in §8 and does not replace them; each plan stays the authority for its
own phases and state. Rulings new here: who announces you (§6, 2026-09-30),
and on 2026-10-01 the three seams this document first left open, WebFinger,
and chatelaine's CXF import policy (§7).
**Audit base:** Mere `3fd2b147` (2026-09-30), which includes gaz M2's first
intake slice (`c388babb`).

Identity is fundamental to the stack: every port that signs, seals, admits,
syncs or addresses someone asks this tier who is acting and who is on the other
end. Until now the answer has been spread over a tier plan, a research brief,
two founding plans, a proofs plan and three READMEs, each correct about its own
piece. This document is the answer in one place, with the invariants that make
it hold.

---

## 1. The cast

*Dramatis personae*: the persons of the drama. The tier holds both sides of
identity: your faces, and the other players.

| Piece | Kind | Holds | Side |
|---|---|---|---|
| [personae](https://crates.io/crates/personae) | crate | the master keypair, per-protocol derivation, vault, sealed records, the carry model, issuing | me |
| [insigne](https://crates.io/crates/insigne) | crate | proofs: typed keys, delegation certificates and revocations, derived-key attestations, as plain data; checking behind `verify` | between |
| [chatelaine](https://crates.io/crates/chatelaine) | crate (reservation; ruled a plain taxonomy, §7) | secrets: passwords, 2FA seeds, tokens, foreign key material | me |
| [gaz](https://crates.io/crates/gaz) | crate | stored contacts: anchored records, per-endpoint trust, kith and kin, retained proofs | them, kept |
| [castellan](https://crates.io/crates/castellan) | port | guards and presents you: the secret-free views, and the authority that exercises secrets and signs | me, outward |
| [gazette](https://crates.io/crates/gazette) | port | the directory: resolves them, reads what they publish, and (§6) announces you | them, inward; you, outward |
| [dramatis](https://crates.io/crates/dramatis) | crate (reservation; ruled a facade, §7) | the one dependency sibling repos pin for the tier | — |
| `mere-persona-picker` | crate | the Cambium view over the roster | me |

All live in this repository: the crates under `crates/dramatis/`, the ports at
`ports/castellan` and `ports/gazette`. The resident that runs the authority
halves is djinn (`ports/djinn`), which owns one `CastellanResident` as the
single record authority behind every Castellan view
(`ports/djinn/src/resident.rs`). Gazette's authority half joins it as a
djinn-composed service (ruled 2026-10-01, §7; unbuilt).

*2026-10-06 note:* the [dramatis repo plan](../implementation_strategy/2026-10-06_dramatis_repo_plan.md)
(rulings D1 to D3) moves the tier to its own repository, folds chatelaine
into dramatis as its keychain, and narrows castellan to the half that
holds and exercises secrets, personae's vault custody included. Nothing
has moved yet; this section describes the tree as it stands.

## 2. Three axes

The tier is organised along three independent axes. Most of its boundaries are
one of these, and a proposed change that blurs one deserves a ruling.

**Me and them.** personae is the *me* side and gaz the *them* side. The two
were never one crate, and gaz as an umbrella over the tier was declined
(2026-08-10) because "the 'them' side owning the 'me' side inverts the model".
castellan faces outward from *me*; gazette faces inward toward *them*. The
contact brief's founding pair (the persona brief for me, the contact brief for
them) is the same split.

**Shown and exercised.** insigne holds what is made to be shown, chatelaine
what must never be. The line is cryptographic, not filing: an insigne is a
public-key artifact that survives disclosure (a signature reveals no key); a
chatelaine item is bearer or symmetric material that disclosure damages
(showing a password burns it, showing a TOTP seed clones it). An insigne is
the interchange artifact between two people's tiers: "your insigne is what
someone else's gaz keeps" (Mark, 2026-08-10).

**Embeddable and authority.** Each port splits in two. The embeddable half is
views any host composes; they render *about* secrets and never contain them.
The authority half lives with the resident and answers petitions. A host that
embeds a view and is later compromised can lie about labels; it cannot
exfiltrate a secret, and the consent prompt renders on the resident's surface,
not the requester's. This is the ssh-agent's shape generalised: apps talk to a
pipe and never see the key.

Under all three sits the **two-plane model** from the personae founding: the
tier is the trust plane and holds keys and trust, never the bytes they seal.
Persistence is the eidetic family (muniment, chartulary). The planes bond at
two seams: the **seal seam** (personae owns the epoch key; pandect's
`WalletEpochSealer` is the joint) and the **sync gate** over codicil.

## 3. One identity across two people

The lifecycle that the pieces exist to serve, end to end. Steps marked
*unbuilt* are planned and not yet code.

```
 ALICE'S TIER                                         BOB'S TIER
 personae   derive a persona; master key -> per-protocol keys
    |
 castellan  issue an insigne for an audience, at a chosen grade
    |                                   (authority half, in djinn)
    +--> handed: paste, QR, misfin ------------------+
    |                                                |
 gazette    announce it at the persona's handle (§6) |
            (well-known JRD, nostr.json, card)       |
            ...............................resolve <-+--- gazette  (Bob's)
                                                     |
                                              intake (gaz M2)
                                                     |
                                                    gaz  keep record + proof
                                                     |
                                     re-check later (insigne `check`,
                                     newer revocations via notochord)
```

1. **A persona is derived.** personae's `IdentityProvider` derives each
   protocol's key from the master seed (BLAKE3-keyed over a salt) and can
   attest it with a `DerivedKeyAttestation`. Issuing stays in personae, as the
   `Issue` and `AttestationKeys` traits (`crates/dramatis/personae/src/delegation.rs`,
   `provider.rs`).
2. **An insigne is issued for an audience.** Grades run from a bare key
   (continuity), through signed claims binding handles and endpoints to a
   persona key, through delegation certificates, to chain-root linkage. The
   grade is chosen per audience; an insigne need not be the most stringent
   proof available. castellan's authority half signs.
3. **It travels by one of two carriages.** Handed bilaterally (paste, QR, a
   misfin message; hocket's contact token is a working v0 insigne), or
   announced at a handle for anyone holding that handle to resolve (§6;
   *unbuilt*).
4. **The other side resolves it.** gazette turns a handle into typed
   endpoints. WebFinger is built; the key-returning resolvers (atproto
   `did:plc`, NIP-05) are *unbuilt*, and checking PLC operations is gazette's
   job, not gaz's (gaz founding plan, M2).
5. **Intake files it.** gaz M2: resolver output becomes a record under an
   anchor, with each key's `ProofMethod` and the proof artifact itself
   retained (`KeyProof`). Unverified address intake from WebFinger landed
   2026-09-30 (gazette's `intake::WebFingerIntake` into
   `ContactBook::intake_addresses`); checked key and PLC intake are
   *unbuilt*.
6. **It is checked again later.** A retained artifact can be re-checked
   against a newer revocation list, which is why gaz keeps the proof and not
   only its name.

Secrets take a different path, and never this one: a chatelaine item is
exercised by castellan's authority half on petition (the OTP release gate is
the built example, `ports/castellan/src/otp/release.rs`) and its *result*
reaches the host, never the item.

## 4. Invariants

What makes the model hold, each with where it is enforced. A change that
breaks one is an architecture change and comes to Mark first.

1. **Hosts never hold secrets.** castellan's `view` is the secret-free read
   model; the secrets stay in the authority (`PersonaeHost`,
   `ports/castellan/src/authority.rs`). Approval renders on the resident's
   surface.
2. **The contact store carries no cryptography.** gaz never verifies a
   signature. Its only runtime dependency in the tier is insigne's plain core;
   personae and insigne's `verify` are dev-dependencies
   (`crates/dramatis/gaz/Cargo.toml`). gaz records which proof the caller
   checked; it does not check.
3. **A passing check is a local conclusion, never data.** insigne's `check`
   returns a `CheckedAttestation`, `CheckedCertificate` or `CheckedRevocation`
   that borrows its statement, has private fields and is not serializable, or
   a `CheckFault` naming the failed step (`crates/dramatis/insigne/src/`).
   Only a passing check can make one. A serialized view may carry the
   *outcome*, never the conclusion (castellan's `DeviceGrantView`, confirmed
   by Mark 2026-09-26).
4. **Issuing is personae's; checking is insigne's.** Issuing needs the
   persona's keys; checking needs only public data. The split is what lets a
   holder like gaz depend on the proof types without the keys.
5. **A contact is anchored, never rooted on a handle.** The anchor is a typed
   key, a `did:plc`, or a local id (`Anchor`, `crates/dramatis/gaz/src/anchor.rs`).
   It never moves; rotations grow the root line; derived and device keys are
   held concurrently under the root that attested them (`root()`,
   `keys_for(scope)`). WebFinger output never becomes an anchor: a handle
   labels a record and never roots it.
6. **Persona scope is by construction.** One contact book per persona,
   carrying its own scope label, so a mis-filed load fails loudly through
   `verify_scope` (`crates/dramatis/gaz/src/book.rs`) rather than showing the
   work persona's people to a burner.
7. **Trust is per address; closeness is per relationship.** `TrustState` is
   held per endpoint and per handle binding. Kith and kin are set by the user
   and never derived from verification, so a peer does not become close
   because a certificate checked out.
8. **A mismatch is never resolved silently.** A pin that stopped matching is
   either an unannounced rotation or someone in the middle; `Mismatched` and
   `Revoked` are `is_alarming()` and surface in `alarms()`. Intake reports a
   key change without proof as `Mismatched`.
9. **Persona separation is the reveal gradient.** Gazette needs no
   authenticated resolution: each persona has its own handle, and each handle
   resolves anonymously to what that persona reveals. Who holds which handle
   is the tiering. Genuinely gated reveal is murm, misfin and moot territory.
   The limit is stated honestly: an unshared handle is a bearer capability
   guarded by obscurity.
10. **The tier holds keys and trust, never the bytes they seal.** At-rest
    sealing of a gaz book is the host's, through castellan's
    `PersonaeHost::sealed_backend` and pandect's wallet backend; gaz stays
    plain.
11. **No clock in the model.** gaz takes every timestamp from the caller, so
    recency is monotonic and a replayed event cannot rewind a record.
12. **An import never widens what is exercised without the user.** A
    quarantined item is sealed and is never filled, released or shown until
    the user accepts it into the vault, one item at a time (rulings 11, 13
    and 15, §7). Ruled 2026-10-01; unbuilt, so not yet enforced in code.
13. **While locked, no secret is reachable, and unlocking takes a user act
    on the resident's own surface.** Enforced in four places:
    - the vault's lock state: personae's `IdentityVault::lock` drops the
      profile and storage key, and its accessors return `Locked`
      (`crates/dramatis/personae/src/vault.rs`);
    - castellan's `ResidentLock`, whose synchronous holder hooks drop
      every derived key before `lock()` returns
      (`ports/castellan/src/authority.rs`);
    - the persisted lock: personae's startup loaders refuse an unattended
      open under the marker (`crates/dramatis/personae/src/startup_unlock.rs`);
    - djinn's triggers: session lock, suspend and idle
      (`ports/djinn/src/lock_triggers/`).

    What stays while locked, and what the lock does not defend, are in the
    [vault threat statement](2026-10-08_vault_threat_statement.md). Added
    2026-10-08 by the
    [vault lock plan](../implementation_strategy/2026-10-05_vault_lock_plan.md)'s
    ruling 83.

## 5. What each side is for

Read together, the two ports are the tier's whole outward face, pointing in
opposite directions over one spine:

- **castellan carries authority outward**: it proves, signs and releases, for
  you, on petition.
- **gazette carries knowledge inward**: it resolves, reads, and hands what it
  learns to gaz.

Both are persona-scoped, both compose gaz, and both put their sensitive half
with the resident. Ruling 3 (§6) adds the one place where gazette also faces
outward.

## 6. Ruling: who announces you

**The gap.** Gaz M2 raises a handle binding by *back-claim*: the handle's own
well-known document names the key or DID. That presumes someone serves the
document. For other people, their host does. For your own personae, nothing in
the tier did: castellan signs presentations, gaz builds the public card
(`publish(&PublicCard)`, gaz's `jscontact` feature), gazette resolves *other*
people's. Neither the contact brief, the personae founding, nor djinn's
resident plan named an owner.

**Ruled 2026-09-30 (ruling 3, §9): castellan issues, gazette announces.**
castellan's authority half signs the presentation and picks its grade per
persona; gazette serves what it is given, so the directory is two-way: it
gazettes you and resolves them. The name already said so: to be *gazetted* is
to be officially announced and thereby resolvable.

What follows, as *Reading, not ruled*:

- gazette announces only what castellan has issued. It holds no key and signs
  nothing, so invariant 1 holds unchanged: the announcing surface is public by
  construction, and the secret-holding port grows no public listener.
- Announcing is per persona and never crosses them. One persona's document
  must not name another's handle or key, or invariant 9's gradient collapses.
  The reading-identity concern from the credential brief (fetching a friend's
  feed reveals your interest to their host) has an announcing mirror: which
  persona's network face serves a document is a setting, not an afterthought.
- gazette's README boundary "gazette reads what is already public" becomes
  "gazette reads what is already public, and announces what castellan has
  issued". Carried into gazette's, castellan's and the dramatis reservation's
  READMEs, the tier plan and the credential brief on 2026-09-30.
- *Where* the announcing process runs was left open by this ruling and
  settled the next day by ruling 6 (§7): a djinn service, exporting static
  files first.

## 7. The seams, resolved

This section named three seams as open on 2026-09-30 (ruling 5). Mark
resolved them on 2026-10-01, together with WebFinger, which ruling 5 had left
out, and the CXF import policy that founding chatelaine raised. All of it is
unbuilt; the rulings say where the work goes. *Amended 2026-10-02*: chatelaine
is built through its plan's P3 (the taxonomy, castellan's OTP items and its
Secret Service store on it); CXF import, the gazette service, WebFinger's move
and the facade remain unbuilt.

**Gazette's authority half is a djinn service (ruling 6).** Djinn composes it
beside `CastellanResident`, on the djinn plan's own split: djinn owns process
lifetime and scheduling, gazette owns resolution, contact intake and
announcing policy. Feed polling becomes a job on djinn's resident scheduler,
whose extraction that plan wants driven by real jobs. Announcing exports
static files first: gazette writes a persona's well-known documents (the
WebFinger JRD, `nostr.json`, the JSContact card) for any HTTPS host to serve.
That order is forced: a WebFinger document must be served over HTTPS at the
handle's own domain, and djinn's resident site service
(`ports/djinn/src/resident_site.rs`) binds loopback Gemini only today. When
that service gains public binding, it carries the same documents as a
published snapshot. *Reading, not ruled*: a static file answers every
`resource` query with one document, which suits a single-persona domain; a
domain announcing several personae needs a server that reads the query, which
is the djinn listener's job once it exists.

**Gazette's WebFinger moves to finger-protocol (ruling 9).** Compared
2026-10-01:

| | finger-protocol `webfinger` (smolweb) | gazette's own |
|---|---|---|
| I/O | none: `request_url` and `parse`; the caller does the GET | `reqwest::blocking` inside the fetch functions |
| wasm32 | compiles (checked 2026-10-01) | cannot: reqwest 0.12.28 compiles `blocking` out on wasm32 (its `if_hyper!` gate) |
| JRD model | all of RFC 7033 §4.4, nullable properties and link titles included | requires `subject`; keeps a link's `rel`, `type` and `href` only |
| direction | parses and serializes, which announcing needs | parses only |
| resources | `acct(user, host)`; the caller supplies the host | normalizes a bare `user@host`, an `acct:` URI or a URL, origin and port included |

Gazette adopts finger-protocol for the request URL and the JRD in both
directions, keeps its endpoint classification and intake (domain logic, not
wire code), and drops the blocking fetch: the caller supplies HTTP. Gazette's
resource normalization moves upstream into finger-protocol in smolweb, so the
spec crate carries all of the wire. errand still takes finger-protocol without
`webfinger`, because errand does not speak HTTP (smolweb's home decision).

**chatelaine is a plain taxonomy crate (ruling 7).** Shaped like insigne's
core: CXF-shaped item kinds and secret-free metadata (ids, labels, persona
scope, origin), with no secret bytes, no storage and no cryptography.
castellan's `OtpItemId` and its `OtpItem` read model move in; the sealed
store, the release gate and exercising stay in castellan, as the crate
consolidation plan established on 2026-09-23. Import policy for CXF v1.0's 17
credential types (Proposed Standard, 2026-03-09), rulings 10 to 15:

| CXF types | On import |
|---|---|
| basic-auth, generated-password, totp, api-key, wifi | stored as chatelaine kinds; totp onto castellan's existing RFC 6238 items |
| ssh-key | through castellan's native SSH import (`ImportSshKeyNativeIntentV1`) into personae's SSH slots, so SSH has one home |
| address, person-name | stored as chatelaine autofill kinds, exercised only by filling forms |
| note, custom-fields | stored as chatelaine kinds |
| item-reference | kept as a link between items, not an item; a dangling one is reported |
| passport, drivers-license, identity-document, credit-card | quarantined |
| passkey, file | quarantined until a passkey provider or blob custody exists |
| types newer than CXF v1.0 | quarantined, original fields preserved verbatim |

Quarantine, as ruled: sealed on import, never exercised or autofilled,
listed for review, and accepted into the vault or deleted by the user one
item at a time (invariant 12). CXP is still a working draft, so a `.cxf`
file is plaintext on disk, and every import path treats it as burning
(standards survey §2.3). The work, and rulings 16 to 22 on the item model,
the Secret Service store, the parser, live data, personae per account and
export, are in the
[chatelaine and CXF plan](../implementation_strategy/2026-10-01_chatelaine_cxf_plan.md).

**The dramatis facade is real, for sibling repos (ruling 8).** `dramatis`
re-exports personae, insigne and gaz behind features, so a repo outside mere
pins one crate at one revision. The evidence, 2026-10-01: eight sibling repos
consume the tier at four mere revisions (`d82afa17`: mer3ly, hocket and
retinue's signalman desktop; `8106c7c2`: cleromancy and woodshed;
`bd5912fb`: turnstone and knot-editor; `32edc2ad`: isometry). Turnstone and
knot-editor match personae and insigne to one revision by hand, the hazard
the insigne proofs plan's phase C handoff spells out. Mere's own crates keep
their direct dependencies. *Reading, not ruled*: the facade covers the
crates, not the two ports, which hosts compose rather than import as a tier;
chatelaine joins it once founded.

## 8. Where each piece's state lives

This document carries no phase status, which goes stale; the plans carry it.

| Piece | Authority |
|---|---|
| the tier's founding and the port direction | [dramatis tier plan](../../archive_docs/2026-10-06_completed_plans/2026-08-10_dramatis_tier_plan.md) |
| the credential port and gazette fronts, standards inventory | [credential port + gazette brief](../../mere_docs/research/2026-08-10_credential_port_gazette_brief.md) |
| personae's scope and seams | [personae founding](2026-07-08_personae_founding.md) |
| delegation grammar's content | [device-grant delegation reconciliation](../../mere_docs/technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md) |
| insigne's move and API | [insigne proofs plan](../../archive_docs/2026-10-06_completed_plans/2026-09-23_insigne_proofs_plan.md) |
| the contact model (the *them* spec) | [contact identity model brief](../../mere_docs/research/2026-06-15_contact_identity_model_brief.md) |
| gaz's phases, anchors, intake | [gaz founding plan](../implementation_strategy/2026-08-08_gaz_founding_plan.md) |
| castellan's keeper surface | [castellan keeper founding plan](../../archive_docs/2026-10-06_completed_plans/2026-08-14_castellan_keeper_founding_plan.md) |
| castellan's credential runway | [castellan OTP plan](../../archive_docs/2026-10-06_completed_plans/2026-08-10_castellan_otp_plan.md) |
| the vault's lock | [vault lock plan](../implementation_strategy/2026-10-05_vault_lock_plan.md) |
| what the lock defends and leaves open | [vault threat statement](2026-10-08_vault_threat_statement.md) |
| the resident | [djinn resident services plan](../../mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md) |
| chatelaine, insigne, dramatis as crates | [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md) |
| standards grading (JSContact, DIDs, CXF) | [standards survey brief](../../2026-08-24_standards_survey_brief.md) |

## 9. Rulings

Mark's answers on 2026-09-30 (rulings 1 to 5) and 2026-10-01 (6 to 15), from
multiple-choice rounds. Each answer was an option label, quoted verbatim.

**Ruling 1.** *What should thinking about dramatis / castellan / gazette
produce first?* Options: gazette's next plan; a seam design round; a
whole-tier architecture brief; consumer convergence. Mark: **"Whole-tier
architecture brief"**. Follows: this document.

**Ruling 2.** *Should announcing you be in scope for this thinking?* Options:
yes, rule its owner; not yet. Mark: **"Yes, rule its owner (Recommended)"**.
Follows: ruling 3.

**Ruling 3.** *Who owns announcing you: serving a persona's own well-known
documents (WebFinger JRD, nostr.json, a JSContact card) so that other
people's gazettes can resolve it?* Options: castellan issues, gazette
announces; castellan does both; the host serves it. Mark: **"Castellan
issues, gazette announces (Recommended)"**. Follows: §6.

**Ruling 4.** *Where does the brief go?* Options: write now at
`dramatis_docs/technical_architecture/` and index after the peer session
commits; write now and message the peer to carry the index line; a research
brief instead. Mark: **"Write now, index after peer (Recommended)"**.
Follows: this file's location. The peer's M2 work landed as `c388babb`
while this document was written, and its index line and D2 record followed
in the same session.

**Ruling 5.** *Which open seams should the brief name as open questions?*
Options (multi-select): gazette's resident slot; WebFinger done twice;
consumer version spread; chatelaine and the facade. Mark: **"Gazette's
resident slot, Chatelaine and the facade"**. Follows: §7 names those two.
*Reading, not ruled*: the two unselected items are left out of this document
rather than declared settled. The consumer version spread is already
recorded in the insigne proofs plan's phase C handoff. *Amended 2026-10-01*:
Mark then asked for the WebFinger comparison ("i imagine the sans-io version
is more portable... shall we compare the webfingers?"), which ruling 9
settles, and the facade ruling (8) answers the version spread for sibling
repos.

**Ruling 6.** *Where does gazette's authority half (polling, trust intake,
announcing) live?* Options: a djinn service, exporting static files first;
its own resident process; in the host, with no resident. Mark: **"Djinn
service, static export first (Recommended)"**. Follows: §7.

**Ruling 7.** *What is chatelaine as code?* Options: a plain taxonomy crate;
a castellan module; stay reserved. Mark: **"Plain taxonomy crate
(Recommended)"**. Follows: §7, and the CXF import rounds (rulings 10 to 15).

**Ruling 8.** *What becomes of the dramatis facade?* Options: a real facade
for sibling repos; retire the reservation; keep it empty. Mark: **"Real
facade for sibling repos (Recommended)"**. Follows: §7.

**Ruling 9.** *Should gazette adopt finger-protocol's sans-io WebFinger?*
Options: adopt and move gazette's resource normalization upstream into
finger-protocol; adopt and keep normalization in gazette; keep gazette's own.
Mark: **"Adopt; upstream normalization (Recommended)"**. Follows: §7; the
upstream move touches smolweb, which Mark brought into scope with this
answer.

**Ruling 10.** *CXF core credentials (basic-auth, generated-password, totp,
api-key, wifi, ssh-key): how are they imported?* Options: store, with ssh-key
through castellan's SSH import; store all six as chatelaine kinds; store
five and drop wifi. Mark: **"Store; ssh-key via SSH import (Recommended)"**.

**Ruling 11.** *Identity documents and payment cards (passport,
drivers-license, identity-document, credit-card): store, quarantine or
drop?* Options: quarantine; store as chatelaine kinds; drop with a report.
Mark: **"Quarantine (Recommended)"**.

**Ruling 12.** *Autofill personal data (address, person-name): where does it
go?* Options: chatelaine autofill kinds; drafts on the persona's JSContact
card; drop with a report. Mark: **"Chatelaine autofill kinds
(Recommended)"**.

**Ruling 13.** *Types castellan can't use yet, passkey (no WebAuthn provider
exists) and file (castellan holds no blobs): what happens on import?*
Options: quarantine both; drop with a report; quarantine passkey and drop
file. Mark: **"Quarantine both (Recommended)"**.

**Ruling 14.** *Freeform types (note, custom-fields, item-reference): how
are they imported?* Options: notes and fields stored with references kept as
links; all three stored as kinds; notes and fields quarantined. Mark:
**"Notes and fields stored; references as links (Recommended)"**.

**Ruling 15.** *Credential types CXF adds after v1.0, which this importer
won't recognize: what happens?* Options: quarantine with original fields
preserved verbatim; drop with a report. Mark: **"Quarantine, preserved
verbatim (Recommended)"**.

Rulings 10 to 15 answer the standards survey's open decision 5 (CXF import
policy), which had been open since 2026-08-24. Rulings 16 to 22, the same
day, belong to chatelaine's implementation and are recorded in the
[chatelaine and CXF plan](../implementation_strategy/2026-10-01_chatelaine_cxf_plan.md) §2.

*2026-10-05:* the stack seams plan's ruling S9 (Mark: "mien's becomes
PersonaKey (Recommended)") renamed mien's `PersonaId`, a persona's leaf
public key, to `PersonaKey`, so `PersonaId` names only personae's persona
UUID. Carried by this tier's lane as `f702ca27` (merged `b52edea7`); no
wire or persisted format carried the type. The vault's lock, found missing
on 2026-10-04, has its own [vault lock plan](../implementation_strategy/2026-10-05_vault_lock_plan.md).
