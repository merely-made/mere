# Dramatis Tier Architecture

**Date:** 2026-09-30
**Status:** current. The tier's architecture of record: one account of identity
across the six crates and two ports that carry it. It synthesises the plans
listed in §8 and does not replace them; each plan stays the authority for its
own phases and state. One ruling is new here (§6, who announces you).
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
| [chatelaine](https://crates.io/crates/chatelaine) | crate (reservation) | secrets: passwords, 2FA seeds, tokens, foreign key material | me |
| [gaz](https://crates.io/crates/gaz) | crate | stored contacts: anchored records, per-endpoint trust, kith and kin, retained proofs | them, kept |
| [castellan](https://crates.io/crates/castellan) | port | guards and presents you: the secret-free views, and the authority that exercises secrets and signs | me, outward |
| [gazette](https://crates.io/crates/gazette) | port | the directory: resolves them, reads what they publish, and (§6) announces you | them, inward; you, outward |
| [dramatis](https://crates.io/crates/dramatis) | crate (reservation) | the tier facade, if one earns its existence | — |
| `mere-persona-picker` | crate | the Cambium view over the roster | me |

All live in this repository: the crates under `crates/dramatis/`, the ports at
`ports/castellan` and `ports/gazette`. The resident that runs the authority
halves is djinn (`ports/djinn`), which owns one `CastellanResident` as the
single record authority behind every Castellan view
(`ports/djinn/src/resident.rs`).

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
- *Where* the announcing process runs (djinn, a separate listener, an
  external web host the user already has) is not decided by this ruling. It
  falls inside the first open seam below.

## 7. Open seams

Named, not resolved (ruling 5).

**Gazette's resident slot.** gazette's README puts its authority half
(resolution, feed fetching, trust state) "with the resident, which is the
always-on party and therefore the natural poller". djinn's resident services
plan names Personae and Castellan as "local caller identity and durable secret
authority" and has no slot for gazette. With ruling 3, the slot now has three
jobs: polling, trust state, and announcing. Blocking `reqwest` in gazette is
the known entry ticket (the credential brief's first open question for the
gazette).

**chatelaine.** Still a reservation. The crate consolidation plan rules it
"design first": a secret-item taxonomy shaped against CXF's credential kinds,
which castellan's secret-free OTP item types move into once it exists.
castellan's sealed OTP store stays in castellan either way, because it is
castellan's code.

**The dramatis facade.** Still a reservation, and it stays empty until
something imports it. Nothing found as of this date wants one: every
consumer names the member crates directly.

## 8. Where each piece's state lives

This document carries no phase status, which goes stale; the plans carry it.

| Piece | Authority |
|---|---|
| the tier's founding and the port direction | [dramatis tier plan](../../mere_docs/implementation_strategy/2026-08-10_dramatis_tier_plan.md) |
| the credential port and gazette fronts, standards inventory | [credential port + gazette brief](../../mere_docs/research/2026-08-10_credential_port_gazette_brief.md) |
| personae's scope and seams | [personae founding](2026-07-08_personae_founding.md) |
| delegation grammar's content | [device-grant delegation reconciliation](../../mere_docs/technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md) |
| insigne's move and API | [insigne proofs plan](../implementation_strategy/2026-09-23_insigne_proofs_plan.md) |
| the contact model (the *them* spec) | [contact identity model brief](../../mere_docs/research/2026-06-15_contact_identity_model_brief.md) |
| gaz's phases, anchors, intake | [gaz founding plan](../implementation_strategy/2026-08-08_gaz_founding_plan.md) |
| castellan's keeper surface | [castellan keeper founding plan](../../mere_docs/implementation_strategy/2026-08-14_castellan_keeper_founding_plan.md) |
| castellan's credential runway | [castellan OTP plan](../../mere_docs/implementation_strategy/2026-08-10_castellan_otp_plan.md) |
| the resident | [djinn resident services plan](../../mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md) |
| chatelaine, insigne, dramatis as crates | [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md) |
| standards grading (JSContact, DIDs, CXF) | [standards survey brief](../../2026-08-24_standards_survey_brief.md) |

## 9. Rulings

Mark's answers on 2026-09-30, from multiple-choice rounds. Each answer was an
option label, quoted verbatim.

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
recorded in the insigne proofs plan's phase C handoff.
