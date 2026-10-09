# Dramatis Repo Plan

**Date**: 2026-10-06
**Status (2026-10-09)**: assessed; rulings D1 to D34 (§3). DR-A landed on 2026-10-09 (§6); DR-B is next.
D18's condition is met: the vault lock's L2 to L4 landed on 2026-10-08.
DR-A's assessment is being refreshed against today's code before its
forks are put to Mark (§6).
**Scope**: the identity tier leaves mere for its own repository, `dramatis`,
which becomes the product. personae, insigne, gaz, chatelaine (the
keychain, still its own crate) and notochord move there, and gazette later.
castellan narrows to the half that holds and exercises secrets. It takes
personae's and pandect's secret custody, stays in mere beside djinn, and
only djinn links it. Every app calls djinn rather than opening a vault.

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
  the workspace manifest depends on it. *Corrected 2026-10-09 (DR-A's
  reassessment):* castellan has depended on chatelaine since `71a91267`
  (2026-10-01).
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

**2026-10-06, the assessment** (a read-only Opus lane at `dbad27d5`; my
spot checks confirmed the claims I re-read: graphshell's default `native`
feature links castellan; pandect's own DPAPI-rooted seed store; gazette's
genet and Cambium dependencies; the picker's lack of consumers; the
port-to-crate boundary script; the live hocket in woodshed):
- **personae's key primitives are everywhere.** 19 mere crates and 8 sibling
  repos use `InMemoryProvider`, `Ed25519Keypair`, `IdentityProvider` or
  `delegation::Issue`, so only custody can move. That is about 5,400 lines
  (the vault, storages, unlock ladder, Hello gate, roster opens), plus
  about 3,700 for the SSH agent and CA. `SealedRecordStorage` is a keyed
  primitive that four production crates use with keys they derive
  themselves.
- **pandect holds a second custody ladder**
  (`wallet_store/secrets.rs:82-250`, `devices.rs`, `epochs.rs`). Knot's
  and Retinue desktop's seeds come from it. As a crate, pandect may not
  depend on a port (`scripts/check_port_boundaries.py`).
- **castellan's cut.**
  - The authority, items, the Secret Service, the record store and the
    secret half of OTP are secret-bearing.
  - `view.rs`, `projection.rs` and the OTP tiles and credentials are
    secret-free. But `view.rs` loads from pandect, and the renderer needs
    chirograph.
  - `personae::signing` mixes secret-free request records with the
    approval broker.
- **graphshell links castellan in every default build**, so Turnstone does
  too. About 3,300 lines of graphshell hold the authority or the vault.
  `mere-signalman` links castellan for its station grants.
- **No djinn route** derives, attests, issues, signs or seals for an app
  today. The app id is "a *label*, not a credential"
  (`app_admission.rs:16-22`).
- **gazette** needs five mere crates and two genet crates. **The persona
  picker** needs ten Cambium-family crates and has no consumers. Moving
  either would make mere and dramatis pin each other.
- **Two revisions of personae in one graph** split its types. Every
  sibling's dramatis pin must equal the one its mere revision names.

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

Rulings D6 to D21 were asked on 2026-10-06 from the assessment above.

**Ruling D6.** *Which repository holds castellan?* Options: stays in mere
beside djinn (no repo cycle; the check lives in mere; castellan's docs stay
in mere); moves to dramatis (a cycle both ways, the security core in the
product repo). Mark: **"Hm. I suppose i'd be ok with it staying in mere.
But! Notochord, we should consider, no? Doesn't it have an identity
angle?"** Follows: castellan stays in mere; notochord was put as D10.

**Ruling D7** *(amends D1 for gazette).* *gazette needs five mere crates and
two genet crates, and the persona picker ten Cambium-family crates with no
consumers; moving them makes mere and dramatis pin each other, the cycle
the cambium move removed.* Options: the leaf crates first, gazette staying
until its domain logic is split from its Cambium projection, the picker
staying or retiring; everything now with a Knot-style lockstep. Mark:
**"Leaf crates first (Recommended)"**.

**Ruling D8** *(extends D3).* *pandect holds the wallet's identity seed,
device seed and private epochs under its own DPAPI root; Knot's and Retinue
desktop's seeds come from it.* Options: the wallet's secret half moves into
castellan, with pandect keeping the public manifests, roster and grants;
it stays in pandect behind a feature the check bans; record the gap. Mark:
**"Wallet secrets to castellan (Recommended)"**. Follows: vault lock ruling
15 is met the same way once this lands.

**Ruling D9.** *What "personae's secrets" covers.* Options:
- custody and the agent move (vault, storages, unlock ladder, Hello gate,
  roster opens, SSH agent, signing broker), while the primitives, traits,
  issuing, `seal_bytes`, the carry types, the secret-free vault types and
  `SealedRecordStorage` stay;
- the same with the agent staying in personae;
- every secret type, which breaks the protocol stack.

Mark: **"Custody and agent (Recommended)"**.

**Ruling D10.** *Notochord (session admission: delegation chains, owner
policy, revocation ledger) depends only on personae and insigne and holds
no secrets of its own.* Options: moves with the leaf crates; stays in mere
beside the transports. Mark: **"Moves with the leaves (Recommended)"**.

**Ruling D11.** *What djinn hands apps (D5).* Options:
- mixed: root-level acts (sign, issue, attest) happen inside djinn on
  request, ssh-agent style, and only namespaced derived keys an app needs
  continuously (transport, sealing) are released, a lock revoking them
  through the broadcast;
- release derived keys for everything;
- a request for everything.

Mark: **"Mixed (Recommended)"**. *Reading, not ruled:* the release path is
the part nearest our own mechanism; it follows the Secret Service's
release-after-unlock shape.

**Ruling D12.** *An app when djinn is absent or Locked.* Options:
- one rule for every app: identity pending, never a fallback key, never an
  unsealed write, already-public state readable;
- the same plus launching djinn on demand;
- decided per app.

Mark: **"One rule: pending (Recommended)"**. Follows: this generalises
vault lock ruling 35. Turnstone's unsealed fallback and woodshed's
unsealed writes both go.

**Ruling D13.** *hocket's own DPAPI-rooted identity record.* Options: djinn
adopts it into custody with its fingerprint unchanged (its own profile if
kept apart), and the stale standalone `hocket/` repo is retired or
repointed (the live copy is `woodshed/ports/hocket`); keep it apart and
record the gap. Mark: **"djinn adopts it (Recommended)"**. *Reading, not
ruled:* retire versus repoint for `hocket/` is still Mark's.

**Ruling D14** *(amends D2's "views and projection move into dramatis").*
Options:
- the plain types go to dramatis (view structs, intent payloads and names,
  receipts, the signing request and record types split from `signing.rs`,
  OTP tiles), while the functions that need mere stay with the resident;
- all of both, making dramatis pin mere;
- the views stay in mere.

Mark: **"Types to dramatis (Recommended)"**.

**Ruling D15.** *graphshell links castellan in its default build.* Options:
the adapters that hold the authority move into djinn, with graphshell's
endpoint generic over a trait djinn implements and graphshell no longer
naming castellan; a non-default `keeper` feature only djinn enables. Mark:
**"Adapters to djinn (Recommended)"**.

**Ruling D16.** *How D4's check is enforced.* Options: cargo-deny, banning
castellan with djinn as its only allowed wrapper, in mere and in each
sibling; extending mere's `check_port_boundaries.py`. Mark: **"cargo-deny
(Recommended)"**.

**Ruling D17.** *Station identities (mere-signalman's provisioning bins and
Retinue's signalman-desktop).* First put with options:
- `reticulum` and `grant` to personae as issuing, with station seeds
  outside D5 (recommended);
- signalman as a djinn mode;
- a second castellan linker.

Mark: **"Hmmm. Why is 2 not recommended vs 1?"**

The answer: signalman runs on the owner's machine, where djinn runs, so
the doubt about stations running djinn did not apply. The two options
also answer different questions: where the grant code lives, and who holds
the master. Re-asked with options:
- the grant code to personae as issuing, signalman and signalman-desktop
  calling djinn for the derivation and the grant, and the station's key a
  released station-scoped key tied to the expiring grant (D11's mixed
  model);
- signalman's commands folded into djinn's CLI as well;
- record the gap.

Mark: **"Via djinn, code in personae (Recommended)"**.

**Ruling D18.** *The order against the vault lock.* Options:
- finish the lock first (L2, L3, L4), then the restructure inside mere,
  then the app routes, then the extraction;
- restructure right after L2, so L3 writes into castellan once;
- extract right after L2.

Mark: **"Finish the lock first (Recommended)"**. Follows: vault lock
rulings 28, 32 and 39, which name personae as a location, are built where
they say and move with their code in DR-B. *Reading, not ruled.*

**Ruling D19.** *The docs.* Options: each document follows its subject
(castellan's stay in mere; the tier architecture, gaz, insigne, personae's
and this plan go to dramatis, which founds its own `DOC_POLICY.md` and
audit records); the whole root moves. Mark: **"Each follows its subject
(Recommended)"**.

**Ruling D20** *(amends D1's "fold into dramatis").* First put as a form for
a `dramatis::keychain` module. Mark: **"What? A chatelaine is a keychain.
Why do we need to make a module intermediary?"** Follows: the name stays
chatelaine. Re-asked with options: its own crate in the dramatis repo,
re-exported as `dramatis::chatelaine`; folded into the dramatis crate as
its chatelaine module. Mark: **"Its own crate (Recommended)"**.

**Ruling D21.** *What the keychain covers.* Options: the item types plus
the keychain's secret-free views (item cards, OTP tiles, release requests,
Secret Service metadata); the types only; also personae's SSH slots shown
as keychain items, one browse surface (CXF ruling 10 already routes
imported SSH keys into personae slots). Mark: **"Also SSH keys as items"**.

**Ruling D22** *(confirms D6; asked 2026-10-07).* *Notes Mark pasted on
2026-10-07 proposed the identity repo hold castellan's custody and
authority, with "the identity core and keeper" building and testing on
their own.* Options: D6 stands (castellan in mere beside djinn, the repo
holding only secret-free crates, no cycle); reopen, moving castellan to
dramatis once its pandect and chirograph dependencies are cut. Mark:
**"D6 stands (Recommended)"**.

**Ruling D23** *(confirms D18).* *The same notes recommend promoting the
boundary now and extracting once it works on its own; D18 finishes the
vault lock first because L2 to L4 edit the castellan code DR-A moves.*
Options: D18 stands; start DR-A beside the lock. Mark: **"D18 stands
(Recommended)"**.

Rulings D24 to D27 were asked on 2026-10-09 from DR-A's reassessment
(§6), the first of three rounds.

**Ruling D24** *(which crate D14's "dramatis" is).* *The facade
`crates/dramatis/dramatis` exists and is empty. The views need personae's
signing records, so the facade would depend on personae.* Options: the
facade itself; a new plain-types crate below personae, re-exported by the
facade; split by type. Mark: **"The facade itself (Recommended)"**.

**Ruling D25** *(the signing records until DR-B).* *`personae/src/signing.rs:24-218`
sits wholly behind `agent` (tokio, ssh-agent-lib), so castellan's views
drag `agent` in; its approval broker is custody and moves in DR-B (D9).*
Options: the plain records into an ungated personae module, the broker
staying behind `agent`; into the home crate now, which forces a crate below
personae; defer to DR-B. Mark: **"Personae, ungated (Recommended)"**.

**Ruling D26** *(`VaultLockView` and `VaultProtectionView`).* *Since L2,
`VaultLockView` is the type of castellan's lock watch channel
(`authority.rs:244`, `:487`), consumed by djinn's triggers and status
route.* Options: with the views; personae, beside `IdentityVault::is_locked`;
chatelaine. Mark: **"With the views (Recommended)"**.

**Ruling D27** *(callers through the move).* Options: `pub use` shims at
the old paths, retired in DR-C (D15); shims for outside repos only, with
mere's callers moved now; no shims. Mark: **"Re-export shims
(Recommended)"**.

**Ruling D28** *(`SshKeyView` against D21's "SSH keys as items").*
Options: it moves unchanged, and the item cards come as their own step
after DR-A; item cards now, with an exception to the byte-identical check.
Mark: **"Move unchanged, cards later (Recommended)"**.

**Ruling D29** *(the OTP types' construction invariants).* *`OtpCredential::from_item`,
`OtpCodeTile::new` and `OtpReleaseParticipantClaim::admitted` are callable
only by castellan; in chatelaine they would have to be public; no caller
outside castellan names these types.* Options: the display types
(credential, tile, time ring) move with public constructors, while the
participant claim and release request stay in castellan, so only it
grants `AdmittedSession`; all move with hidden constructors; leave `otp/`
for later. Mark: **"Display types move, trust stays (Recommended)"**.

**Ruling D30** *(the Secret Service metadata, with the service parked by
the Secret Service plan's SS8).* Options: it stays with the parked code;
it moves to chatelaine per D21, with a narrowed error and a ThinkPad
receipt under `dbus-run-session`. Mark: **"Move it per D21"**.

**Ruling D31** *(the intent receipts).* Options: the plain receipts move to
the facade and `IdentityIntentError` stays with the authority; both move;
neither in DR-A. Mark: **"Receipts move, error stays (Recommended)"**.

**Ruling D32** *(DR-A's baseline).* *No baseline exists. The snapshot is
serialized pretty (`to_public_json`) and compact (graphshell's endpoint,
whose revision counter depends on the bytes).* Options: hand-built,
deterministic golden snapshots in both forms, committed before anything
moves, with a control that must fail; captured from a live host; both.
Mark: **"Hand-built golden snapshots (Recommended)"**.

**Ruling D33** *(reading "test counts unchanged").* Options: conserved by
name across castellan and the destinations, plus the new golden tests;
tests stay in castellan and run through the re-exports; literal counts.
Mark: **"Conserved by name (Recommended)"**.

**Ruling D34** *(the order of the ready work).* Options: DR-A, then the
vault lock's ruling 44 (Distillery's transport key), then its ruling 91
(Linux's root in the OS keyring); 44 first; ruling 91's assessment
alongside DR-A. Mark: **"DR-A, then 44, then 91 (Recommended)"**.

Still open: whether gazette gets a matching facade name over gaz, the way
chatelaine is the keychain.

## 4. Phases

Drafted from the assessment and D6 to D21; they start after the vault lock
finishes (D18).

- **DR-A — the type split, inside mere.** The plain types leave `signing.rs`,
  `view.rs`, `projection.rs` and `otp/` for their ruled homes (D14, D21),
  with no change in behaviour. Done when:
  - [x] `IdentitySurfaceSnapshot`'s public JSON is byte-identical against
        a fixture;
  - [x] castellan's, graphshell's and djinn's test counts are unchanged.
        *(2026-10-09, conserved by name per D33; §6.)*

  *2026-10-09, DR-A's checkpoints* (from the reassessment in §6 and rulings
  D24 to D33), smallest blast radius first. Each lands only after its gate
  passes in a worktree with its own build directory (the dynamics grammar
  plan's F183).
  - **A0, the baseline. No move.** Done when:
    - golden JSON for hand-built snapshots (fixed ids and times: unlocked
      with pending signing and history, locked and kept, every enum
      variant), in pretty and compact form, is committed and passes (D32);
    - a control with one renamed field fails it;
    - a by-name test inventory is recorded on Windows and Fedora: castellan
      (default, `keeper`, `secret-service`, all features), personae with
      and without `agent`, graphshell's library, djinn and chatelaine (D33).
  - **A1, the intent names and payloads, to the facade** (D24, D27). Done
    when the constant strings are equal, the shims compile every caller
    unedited, and the goldens and inventory hold.
  - **A2, the OTP display types, to chatelaine** (D29). The credential,
    tile and time ring move with public constructors; the participant
    claim and release request stay. Done when the OTP suites and the tile's
    two tests pass by name.
  - **A3, the signing records, ungated in personae** (D25). The broker
    stays behind `agent`. Done when personae builds and tests unchanged
    with and without `agent`.
  - **A4, the views and the snapshot, to the facade** (D24, D26, D28).
    `VaultLockView` goes with them, `SshKeyView` unchanged, and
    `load_carry_view` stays in castellan. Done when:
    - the goldens are byte-identical in both forms;
    - the facade's tree has no castellan, pandect, chirograph or tokio;
    - djinn and graphshell compile unedited.
  - **A5, the receipts, to the facade** (D31). `IdentityIntentError` stays.
  - **A6, the Secret Service metadata, to chatelaine** (D30), with a
    narrowed error. Done when the Linux build and `secret_service_linux`
    pass under `dbus-run-session`.
  - **A7, close.** Done when:
    - the gate passes on both targets, and Knot's lockstep compiles;
    - castellan's `lib.rs` doc, the graphshell shim headers and invariant
      1's wording ("castellan's `view`") are updated;
    - §2's claim that chatelaine has no dependents is corrected (castellan
      has depended on it since `71a91267`), and so is the Secret Service
      plan's line about moving `secret_service/` (D2 keeps the service in
      castellan; D21 and D30 move only its metadata).
- **DR-B — custody into castellan, inside mere** (D8, D9). Done when:
  - [ ] personae's and pandect's dependency trees no longer reach the DPAPI
        root loaders or the passphrase storage (measured with `cargo tree`);
  - [ ] the no-residue instrument passes in its new home, and its positive
        control fails when a fix is reverted;
  - [ ] the cargo-deny check (D16) passes, and fails on a control crate that
        adds castellan;
  - [ ] the gate passes.
- **DR-C — djinn routes and the app migrations** (D5, D11 to D13, D15,
  D17): the graphshell adapters move into djinn. Turnstone, woodshed,
  hocket, the Knot desktop and bins, signalman, the Distillery bin and the
  `personae-vault` CLI call djinn. Done when:
  - [ ] no app opens a vault, storage or wallet (measured by search);
  - [ ] the check passes in every sibling;
  - [ ] each app has a receipt with djinn absent and with djinn Locked,
        showing the identity pending, no fallback key and no unsealed
        write;
  - [ ] a reintroduced fallback fails that receipt.
- **DR-D — extraction** (D7, D10, D19, D20): the repo is created with
  history (the cambium precedent). mere and the siblings repin, the docs
  move, and mere's patch table grows. Done when:
  - [ ] every graph resolves exactly one personae source (measured);
  - [ ] mere's gate and both doc audits pass;
  - [ ] Knot's lockstep run passes, test targets included.
- **DR-E — crates.io metadata and publishing.** Mark's step.

## 5. Findings

**2026-10-06, from the assessment.** Not verified:
- which of these crates are published, and at what versions;
- whether Cargo accepts a `[patch]` for dramatis.git inside mere (a
  same-source refusal is expected);
- whether muniment, gaz's optional dependency, is published;
- the runtime of any route.

The dramatis facade's own docs say "Not a product", which D1 reverses.
personae's `html_root_url` already says 0.1.0 against its 0.2.1.

## 6. Progress

**2026-10-06.** Opened with rulings D1 to D3. Next: a read-only assessment
of §2's open list.

**2026-10-06.** Rulings D4 (castellan a djinn-only crate, after the vault
lock's L2) and D5 (apps unify on calling djinn). The read-only assessment
(Opus) is running.

**2026-10-06.** The assessment reported 14 forks. My spot checks held;
rulings D6 to D21 settle them. D7, D14 and D20 amend D1 and D2. Next: the
vault lock's L2 (D18).

**2026-10-06, tails inherited from the S14 archive pass** (recorded in the
[archived plan tails plan](../../mere_docs/implementation_strategy/2026-07-03_archived_plan_tails_plan.md), "2026-10-06 archive pass"). This plan
now owns:
- **insigne's remaining repins:** Hocket, Woodshed and mer3ly. Turnstone
  repinned in `d6b62ad`. From the
  [insigne proofs plan](../../archive_docs/2026-10-06_completed_plans/2026-09-23_insigne_proofs_plan.md).
  *Reading, not ruled:* these ride DR-D's repins, and Hocket's goes to the
  live `woodshed/ports/hocket` (D13).
- **The `dramatis` facade**, ruled real on 2026-10-01 and unbuilt. From the
  [dramatis tier plan](../../archive_docs/2026-10-06_completed_plans/2026-08-10_dramatis_tier_plan.md). D1 makes it
  the product; DR-D builds it.
- **The wallet carry gaps:**
  - PAKE/QR chrome and transport UI;
  - copy-mode export and import;
  - epoch history beyond the current epoch;
  - a migration pass for cleartext private blobs.

  From the
  [persona wallet carry layer plan](../../archive_docs/2026-10-06_completed_plans/2026-06-25_persona_wallet_carry_layer_plan.md).
  *Reading, not ruled:* these touch pandect's wallet, whose secret half
  moves into castellan under D8, so they are assessed with DR-B.
- **Splitting castellan's `keeper` feature** (from the castellan keeper
  founding plan; the tails plan keeps it there and says this plan "may take
  it"). *Reading, not ruled:* D14 and D15 make the split unnecessary. The
  view types move to dramatis and graphshell stops naming castellan, so no
  consumer needs views from castellan without its authority.

**2026-10-07, notes on promoting the identity family.** They agree with
rulings already made: D7 (leaf crates first, gazette held until its
projection is split), D10 (notochord moves) and DR-D's done-condition
(two apps on one identity revision, no duplicate types). They differ on
castellan's home and on timing; D22 and D23 keep D6 and D18.

**2026-10-09, D18 met; DR-A reassessed first.**
- Mark: **"also, if it's time to promote dramatis, we can do that too"**.
- **D18 is met.** The vault lock's L2 to L4 landed on 2026-10-08 (vault
  lock plan §6). Ruling 44's transport-key switch is still unbuilt, but it
  is not one of L2 to L4's done-conditions, and it touches only
  Distillery, which stays in mere.
- **Why reassess first:** L2 to L4 reworked castellan's `authority.rs`,
  `view.rs` and `projection.rs` (lock state, the Locked card, the Secret
  Service snapshot), which is the code DR-A moves. So the 2026-10-06 map is
  being redrawn read-only before DR-A's forks are asked.

**2026-10-09, DR-A reassessed (Opus, read-only, at `407bdbe9`), and its
forks ruled (D24 to D33).**
- **What moves:** the views and snapshot (`view.rs:26-133`, unchanged since
  2026-09-26), the intent names and payloads (`projection.rs:28-149`, which
  L2 extended with the lock intents), the receipts (`authority.rs:93-211`),
  the OTP display types, the signing records (`personae/src/signing.rs`,
  behind `agent`), and the Secret Service metadata (new on 2026-10-08).
- **What stays:**
  - the cards and renderer, Locked card included, which need chirograph;
  - the kept snapshot;
  - `VaultLockHolder`;
  - `SecretServiceVault`.
- **Callers:**
  - graphshell's glob shims, through which djinn reaches the types;
  - knot-editor's `resident_app_route` test.
  - Turnstone, woodshed, hocket and retinue name none of these types.
- **Evidence against the record,** corrected in A7:
  - §2 says chatelaine has no dependents, but castellan has depended on it
    since `71a91267`;
  - DR-A's file list missed `authority.rs` and `secret_service/`;
  - the Secret Service plan said the service itself moves.

**2026-10-09, DR-A's A0 to A3.** Worktree `mere-dra`, with its own build
directory (F183). The test inventories are in `Code/testing/mere/dra/`.
- **A0, the baseline** (`f469b836`, on main):
  - `ports/castellan/tests/snapshot_golden.rs` pins three hand-built
    snapshots (unlocked with every signing result, locked and kept,
    ephemeral), each pretty and compact. They are built through
    castellan's old paths.
  - The goldens are LF-pinned in `.gitattributes`, and Windows and Fedora
    produce the same bytes.
  - A control that renames one field fails the two snapshots that carry
    it.
  - The by-name inventories, Windows / Fedora:
    - castellan default 67/67, `keeper` 113/110, `secret-service` 85/88,
      all features 133/133;
    - personae 148/146, with `agent` 220/222;
    - graphshell's library 208/207, djinn 155/150, chatelaine 53/53,
      dramatis 0/0.
- **A1, the intents** (`7af0aec1`, on main): `dramatis::intents` holds the
  18 names and nine payloads verbatim, and `castellan::projection`
  re-exports them. The inventories are identical by name on both machines,
  the goldens pass, and djinn and graphshell compile unedited. The lock
  gains five lines of dependency edges.
- **A2, the OTP display types** (gated on Windows): `chatelaine::otp_tile`
  takes `OtpCredential`, `OtpCodeTile`, `OtpTimeRing` and `OtpFields`, with
  public constructors (D29). The tile's two tests moved with it, and every
  other name is unchanged. The lock gains chatelaine's zeroize edge.
- **A3, the signing records** (gated on Windows): `personae::signing` is
  ungated, and its broker and tests moved to `signing/broker.rs`, behind
  `agent`. Paths are unchanged.
  - personae builds and passes without `agent` (148 tests) and with it
    (220).
  - The broker's four tests moved path with their names intact.
  - graphshell's four new `projection_compare` tests came from upstream
    through the rebase, not from DR-A.
- **Rebases:** main moved twice while A0 and A1 waited, once with 253 lock
  lines. Each time `cargo metadata --locked` confirmed the lock, and the
  gate reran on the rebased tip before the push.

**2026-10-09, DR-A done** (A0 to A7 on main, ending `19853594`).
- **A4, the views and snapshot** (`762c2fa8`): `dramatis::view`. The
  goldens are byte-identical in both forms on Windows and Fedora, and the
  facade's tree has no castellan, pandect, chirograph, tokio or
  ssh-agent-lib.
- **A5, the receipts** (`e68854f3`): `dramatis::receipts`, with
  `IdentityIntentError` staying in castellan.
- **A6, the Secret Service metadata** (`a1a7f1b7`):
  `chatelaine::secret_metadata`.
  - Lookups return a narrowed `MetadataLookupError`, and builder methods
    let castellan's store fill the snapshot.
  - The ids keep a public inner field and their serde form.
  - The first Fedora gate failed to compile castellan with
    `secret-service`: the Linux-only D-Bus layer converts errors into
    `SecretDbusError`, which Windows never builds. The fix, a conversion
    beside the existing one, was folded into the unpushed A6.
  - Fedora then passed, and so did `secret_service_linux` (2 of 2) under
    `dbus-run-session`.
- **A7, close** (`19853594`): the gate passes on both targets.
  - Windows: personae, castellan and djinn, with the `locked_restart`,
    `harness` and `lock_agent` receipts and graphshell's 212. The installed
    resident is unchanged.
  - Fedora: the same crates, with castellan under every feature set.
  - Every test name is conserved against A0. The only moves are the tile's
    two tests (to chatelaine) and the broker's four (to
    `signing::broker`), plus four graphshell tests upstream added during
    the work.
  - Knot: djinn builds knot-site through mere's patch table, and
    knot-editor's `resident_app_route` test names only
    `graphshell::identity` and `graphshell::native::personae_host`, which
    the shims keep.
  - The docs and the two record corrections landed with it.
- **Next:** DR-B, custody into castellan (D8, D9), after the vault lock's
  ruling 44 and ruling 91 by D34's order: DR-A, then 44, then 91. DR-B's
  place in that order is not yet ruled.
