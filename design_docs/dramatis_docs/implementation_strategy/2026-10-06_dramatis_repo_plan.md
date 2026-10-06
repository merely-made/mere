# Dramatis Repo Plan

**Date**: 2026-10-06
**Status (2026-10-06)**: opened; rulings D1 to D5 (§3) set the shape. Nothing
assessed in depth and nothing built. Next: a read-only assessment (§2's open
list), then a round of forks for Mark. The build is sequenced against the
vault lock plan, which is editing castellan and personae now.
**Scope**: the identity tier leaves mere for its own repository, `dramatis`,
which becomes the product. personae, insigne, gaz and gazette move there.
chatelaine folds into dramatis as its keychain, as gazette is roughly its
contact list. castellan narrows to the half that holds and exercises
secrets, and takes personae's secret custody with it.

**Related**:

- [dramatis tier architecture](../technical_architecture/2026-09-30_dramatis_tier_architecture.md):
  the architecture of record; its §1 says every piece lives in mere, and its
  invariant 1 ("hosts never hold secrets") is the line this plan makes
  structural.
- [vault lock plan](2026-10-05_vault_lock_plan.md): L1 to L4 change the
  vault, its storages and castellan, the code this plan moves.
- [chatelaine and CXF import plan](2026-10-01_chatelaine_cxf_plan.md):
  chatelaine as a plain taxonomy crate, which D1 folds into dramatis.
- [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md):
  dramatis ruled a real facade for repos outside mere (2026-10-01), unbuilt.

---

## 1. Why

Mark, 2026-10-06: "we were considering promoting the identity stack out of
mere after refactoring its boundaries… but also renaming 'castellan'
'chatelaine' and subsuming the current chatelaine into dramatis…? I think
castellan is taken :/". He then asked: "What if we subsumed both into
dramatis and or personae, made that the product…"

What was checked before the rulings:
- **crates.io.** `castellan` (0.0.3), `chatelaine` (0.0.2) and `dramatis`
  (0.0.2) are all owned by mark-ik.
- **"Castellan" as a business name** is crowded in security. It is used by:
  - a Winnipeg cybersecurity consultancy that holds one registered mark in
    a technology class;
  - a business-continuity software firm acquired by Riskonnect;
  - a small app shop.
  No password manager uses the name.
- **"Chatelaine"** is Canada's largest women's media brand (Rogers Media).
- **"Persona"** is an identity-verification company whose Persona Wallet
  stores identity behind passkeys.
- **"Dramatis"** is otherwise mainly a Shakespeare study app.

So dramatis became the product name. castellan becomes an internal
component, where its crowding matters little.

## 2. What exists (first look, 2026-10-06)

Checked in code at `ea169154`; the assessment goes deeper.

- **Locations.**
  - The crates live under `crates/dramatis/`: personae, insigne, gaz,
    chatelaine, and the dramatis facade (32 lines).
  - The ports are `ports/castellan` and `ports/gazette`.
  - `mere-persona-picker` is the Cambium view over the roster.
  - djinn owns the one `CastellanResident` (`ports/djinn/src/resident.rs`).
- **castellan is 3,626 lines**, splitting roughly as:
  - the secret-free read model and projection hosts render: `view.rs`,
    `projection.rs`, 1,210 lines;
  - the item store: `items/`;
  - one-time codes: `otp/`;
  - the Linux Secret Service: `secret_service/`;
  - `PersonaeHost` (signing, SSH keys, the approval broker, persona
    switching): `authority.rs`, 1,600 lines;
  - the record store djinn owns: `resident.rs`, 218 lines;
  - network grants: `reticulum/grant.rs`.
- **References.** castellan appears in 62 code files, 48 doc files and 605
  lines in mere, plus one file in retinue. chatelaine is 2,206 lines; only
  the workspace manifest depends on it.
- **personae holds secrets today.** The vault, the sealed and passphrase
  storages, the startup unlock ladder and the SSH agent live there, and
  apps link it:
  - graphshell and Turnstone open their own vaults;
  - hocket and woodshed read the storage or the DPAPI root directly.
  That is why vault lock rulings 23 and 36 exist.
- **Invariant 1** (tier architecture §4): "Hosts never hold secrets." Today
  it holds by which process runs what, not by crate.
- **Docs move with code.** Mere's `DOC_POLICY.md` makes it an invariant
  that every area root corresponds to code in this repository, so
  `dramatis_docs/` leaves with the tier. The 2026-09-03 return of three
  roots from genet is the precedent.

Open for the assessment:
- **The cut through castellan:** which modules dissolve into dramatis (the
  views, the projection, the item types) and which stay as castellan.
- **The cut through personae:** which parts move under castellan's custody
  (vault, storages, unlock ladder, agent), and what personae keeps (persona
  types, derivation, issuing).
- **Apps that open the vault themselves** (graphshell, Turnstone, hocket,
  woodshed). If only the resident links castellan, they must reach their
  keys through the resident, not open the vault. This is the largest
  consequence.
- **How mere and its siblings pin the new repo** (git revisions, mere's
  patch table, a lockstep like Knot's), and djinn, which stays in mere and
  links castellan.
- **Where gazette's port and `mere-persona-picker` go.**
- **The crates.io descriptions** (castellan, chatelaine, dramatis,
  personae) and the naming ledger's record.
- **The sequence against the vault lock:** L2 to L4 change castellan, its
  storages and the resident.

## 3. Rulings

These came out of a discussion on 2026-10-06, not a question round. The
questions are as discussed.

**Ruling D1.** *The product and the repo.* Discussed:
- Mark's first idea: castellan renamed chatelaine, with today's chatelaine
  subsumed into dramatis;
- subsuming both into dramatis and/or personae as the product;
- the recommendation: dramatis as the product and repository, with the
  crates kept split by their axis.

Mark: **"Agreed, dramatis as the repo makes sense. Gazette, personae, gaz,
and insigne can go there. Chatelaine can fold into dramatis and be
rethought as its keychain, like gazette would kinda be its contact list. A
simplification but you take my meaning. And then castellan can be split
among the rest as needed?"**

**Ruling D2.** *Where the half that holds and exercises secrets lives.*
Invariant 1 needs that code where only the resident links it. Discussed:
- a crate only djinn depends on;
- a module inside dramatis behind a feature (Mark: "Need it be a crate?
  How about hidden-pocket or something?");
- either way with a check that fails the build if anything else reaches
  it.

Mark: **"Or wait. That secret handling half, can that just be castellan?"**
Follows: castellan keeps its name and narrows to the authority half (items,
one-time codes, the Secret Service, `PersonaeHost`, the record store). Its
views and projection move into dramatis.

**Ruling D3.** *personae's secrets, and opening this plan.* Raised: personae
also holds secrets (vault, storages, agent) and apps link it. Moving that
custody under castellan would make "only castellan touches secrets"
structural rather than a convention. Mark: **"I agree with moving the
personae secrets to castellan and opening this plan, along with that
narrowing of castellan for dramatis."**

*Reading, not ruled* (ruled the same day as D4):
- castellan stays a separate crate, not a feature on an embeddable one. A
  check fails the build if anything but the resident depends on it.
- The build follows the vault lock's L2 at the earliest.

**Ruling D4.** *castellan's form and the sequence.* Put to Mark as the
reading above: castellan stays a separate crate that only djinn links,
with a build check enforcing it, and the move comes after the vault lock's
L2 at the earliest. Mark: **"Agreed!"**

**Ruling D5.** *Apps that open the vault themselves.* graphshell and
Turnstone open their own vaults; hocket and woodshed read the storage or
the DPAPI root directly. Under D4 they cannot link castellan. Mark: **"i
would much prefer graphshell, turnstone, woodshed, knot-editor, etc. all
unify on calling djinn"**. Follows: every app reaches identity and secrets
by calling djinn, not by opening a vault. The assessment sizes the routes
each needs. *Reading, not ruled:*
- "etc." covers hocket, the other app the map found;
- knot-editor is named although it has no production use of
  `IdentityVault`, because its signing seed comes through pandect's wallet
  (vault lock ruling 15), which also belongs behind djinn.

## 4. Phases

Drafted after the assessment and the forks it raises.

## 5. Findings

None yet.

## 6. Progress

**2026-10-06.** Opened with rulings D1 to D3. Next: a read-only assessment
of §2's open list.

**2026-10-06.** Rulings D4 (castellan a djinn-only crate, after the vault
lock's L2) and D5 (apps unify on calling djinn). The read-only assessment
(Opus) is running.
