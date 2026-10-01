# Chatelaine and CXF Import Plan

**Date**: 2026-10-01
**Status (2026-10-01)**: plan. Shape ruled by Mark on 2026-10-01 (rulings 7
and 10 to 15 in the dramatis tier architecture; rulings 16 to 29 below).
Nothing has moved yet. P0 and P1 start together (ruling 26); the run stops
after P3 for Mark's review (ruling 28).
**Scope**: found `chatelaine` as the tier's plain secret-item taxonomy; move
castellan's OTP items and its Secret Service store onto it; then import
(and finally export) the FIDO Credential Exchange Format through castellan.

**Related**:

- [dramatis tier architecture](../technical_architecture/2026-09-30_dramatis_tier_architecture.md)
  §7: chatelaine's shape and the CXF import table, rulings 7 and 10 to 15.
- [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md)
  C5: the Mere 0.4 baseline waits on chatelaine's taxonomy landing.
- [standards survey](../../2026-08-24_standards_survey_brief.md) §2.3: CXF
  ADOPT (import first), CXP WATCH, the plaintext hazard.
- [castellan OTP plan](../../mere_docs/implementation_strategy/2026-08-10_castellan_otp_plan.md):
  the RFC-vector-verified OTP core this plan re-homes, not rewrites.
- [insigne proofs plan](2026-09-23_insigne_proofs_plan.md): the precedent for
  a plain-data core with no cryptography, checked by a wasm build.

---

## 1. What chatelaine is, and is not

**Is**: plain, serializable item metadata: CXF-shaped items, each a titled
container of typed credentials, with scope (sites and apps), tags, a
favorite flag, timestamps and collections; the credential kinds; and the
import disposition the rulings assign each kind. The OTP display enums
(`OtpAlgorithm`, `OtpKind`, `OtpCodeStyle`) move here, since they describe a
totp credential without being one.

**Is not**: secret bytes, storage, sealing, generation or any cryptography.
A chatelaine value can be shown to any host view without harm, which is
what lets castellan's embeddable half render it. The sealed payloads, the
release gate and every exercise stay in castellan (crate consolidation plan,
2026-09-23 limit).

Illustrative only, not compile-ready:

```rust
pub struct Item {
    pub id: ItemId,                  // uuid, no default features
    pub title: String,
    pub subtitle: Option<String>,
    pub scope: Option<Scope>,        // urls, android apps
    pub tags: Vec<String>,
    pub favorite: bool,
    pub credentials: Vec<Credential>,
    pub state: ItemState,            // Vault | Quarantined
}

pub struct Credential {
    pub id: CredentialId,
    pub kind: CredentialKind,        // metadata only, per kind
}

#[non_exhaustive]
pub enum CredentialKind {
    BasicAuth { username: Option<String> },
    Totp { algorithm: OtpAlgorithm, kind: OtpKind, style: OtpCodeStyle, issuer: Option<String>, account: String },
    /* … the other CXF v1.0 kinds … */
    Secret { content_type: String, attributes: BTreeMap<String, String> }, // Secret Service
    Unknown { cxf_type: String },    // newer than v1.0, preserved (ruling 15)
}
```

Which metadata a kind may carry is itself the line between chatelaine and
castellan. Ruled 2026-10-01 (rulings 23 and 24): **identifying fields only**.
Every item shows its title, subtitle, kind, scope, tags, favorite flag and
timestamps; per kind, chatelaine holds only what tells two items apart, and
castellan seals everything else. This is stricter than CXF's own
`concealed-string` line, which leaves document numbers, addresses, names and
note contents displayable.

| Kind | Chatelaine metadata | Sealed by castellan |
|---|---|---|
| basic-auth | username | password |
| generated-password | — | password |
| totp | issuer, account, algorithm, period, digits | secret |
| api-key | username; key type and URL *(reading)* | key, validity dates |
| wifi | SSID; security type *(reading)* | passphrase, hidden flag |
| passkey | rpId, username; user display name *(reading)* | key, credential id, user handle, extensions |
| ssh-key | key type, fingerprint (derived), comment | private key, dates, generation source |
| credit-card | card type, expiry, last four (derived at import) | number, verification number, PIN, full name, valid-from |
| passport, drivers-license, identity-document | document kind, issuing country, expiry; a licence's territory *(reading)* | numbers, names, birth data, sex, nationality, issue data, authority, licence class |
| address, person-name | — | every field |
| note | — | content |
| custom-fields | field labels *(reading)* | field values |
| file | name, size; integrity hash *(reading)* | bytes |
| item-reference | the link itself | — |
| Secret Service secret | label, lookup attributes, content type (the D-Bus spec's secret-free properties) | bytes |
| unknown (newer than v1.0) | the CXF type string | every field, verbatim |

Rows marked *(reading)* apply the ruled principle to fields Mark's answer did
not name; each is open to his correction before P1 lands.

## 2. Rulings

The tier rulings this plan executes are 7 (a plain taxonomy crate) and 10 to
15 (CXF import per type), recorded in the tier architecture. This plan's own,
from Mark's 2026-10-01 rounds; each answer is an option label quoted
verbatim, except ruling 20, which is his free text.

**Ruling 16.** *What is a chatelaine item?* Options: CXF-shaped (a titled
container of typed credentials); one credential per item. Mark:
**"CXF-shaped (Recommended)"**. Follows: §1; an OTP item becomes an item
holding one totp credential.

**Ruling 17.** *How does chatelaine relate to castellan's Secret Service
store?* Options: subsume it in a later phase; subsume it now; keep it
separate. Mark: **"Subsume it now"**. Follows: P3 is part of founding, so C5's
taxonomy condition is met only when the Secret Service runs on chatelaine.

**Ruling 18.** *Where does the CXF parser live?* Options: castellan behind a
`cxf` feature; `crates/import`; its own crate. Mark: **"Castellan, behind a
cxf feature (Recommended)"**. Follows: P4; plaintext goes from the file to a
sealed record inside the authority and never crosses a secret-free crate.

**Ruling 19.** *Which CXF types does the parser use?* Options: Bitwarden's
crate, diffed against the CDDL; our own serde types. Mark: **"Bitwarden's
crate, CDDL-diffed (Recommended)"**. Follows: P0 gates adoption.

**Ruling 20.** *Which existing castellan stores hold real data that must
survive the move?* Options: OTP items; Secret Service collections; neither.
Mark: **"I think the only real stuff is personae and using it to ssh into my
thinkpad and imacs"**. Follows: OTP and Secret Service records make a clean
break, no decoder (DOC_POLICY §3). *Reading, not ruled*: castellan's existing
OTP v1 legacy reader goes too, having no data to read. And personae's SSH
slots are live data, so P4's SSH routing carries a no-disturbance
done-condition.

**Ruling 21.** *How do a CXF file's Accounts map onto personae?* Options: ask
per account; one persona per file. Mark: **"Ask per account
(Recommended)"**. Follows: P4's import is two-step: read accounts, then
commit with a persona per account.

**Ruling 22.** *Is CXF export part of this plan?* Options: last phase of this
plan; a separate later plan; wait for CXP. Mark: **"Last phase of this plan
(Recommended)"**. Follows: P6.

**Ruling 23.** *Where is the line between chatelaine metadata and sealed
secret?* Options: identifying fields only; CXF's `concealed-string` line;
title only. Mark: **"Identifying fields only (Recommended)"**. Follows: §1's
table.

**Ruling 24.** *Once a payment card or identity document is accepted out of
quarantine, what may views show?* Options: type, issuer, expiry and last
four; type and issuer only; nothing beyond the title. Mark: **"Type, issuer,
expiry, last four (Recommended)"**. Follows: §1's table.

**Ruling 25.** *What happens to RSA and ECDSA SSH keys on import?* Options:
quarantine them; teach the agent RSA and ECDSA; drop with a report. Mark:
**"Teach the agent RSA and ECDSA"**. Follows: P4a. *Reading, not ruled*: the
agent is personae's live SSH path, so P4a tests the new agent beside the
installed one and never replaces an installed agent; deployment to each
machine is Mark's step.

**Ruling 26.** *How should implementation lanes run?* Options: one phase at
a time; overlap independent phases; a workflow script. Mark: **"Overlap
independent phases"**. Follows: §6; P0 runs beside P1.

**Ruling 27.** *Which models for the lanes?* Options: Opus for code and
Sonnet for P0; Opus throughout; Sonnet throughout. Mark: **"Opus for code,
Sonnet for P0 (Recommended)"**.

**Ruling 28.** *How far should this run go before stopping for review?*
Options: through P3; P0 and P1, then review; all phases. Mark: **"Through P3
(Recommended)"**.

**Ruling 29.** *Where does P3's Linux receipt run?* Options: the Fedora
ThinkPad over SSH; the Mint machine; Mark runs it. Mark: **"The Fedora
ThinkPad over SSH (Recommended)"**.

## 3. Phases

Each phase lands with its own tests and gates and keeps the workspace green.
The nested-workspace and all-features lessons of the insigne plan (§4 there)
apply: a census or gate proves nothing about a feature or target it did not
build.

- **P0 — the CDDL diff.** `credential-exchange-format` 0.4.0 (MIT, Bitwarden,
  published 2026-06-11, unchanged since the standards survey flagged it as
  tracking the March 2025 review draft) against CXF v1.0 Proposed Standard
  with errata, 2026-03-09. Done when:
  - [ ] every type and field difference is listed in §4 with its CDDL
        reference;
  - [ ] each is classed: harmless, fixable by a local newtype or an upstream
        patch, or blocking;
  - [ ] if anything blocks, the choice between waiting, patching upstream and
        ruling 19's alternative comes back to Mark as a fork before P4.
        P1 to P3 do not depend on P0's outcome; they run after it, in order.

- **P1 — the taxonomy.** `crates/dramatis/chatelaine` gains real code.
  Done when:
  - [ ] items, credentials, collections and links (`item-reference`) exist as
        plain serde types; `CredentialKind` is `#[non_exhaustive]` and covers
        CXF v1.0's 17 kinds, the Secret Service generic secret, and
        `Unknown` with its preserved type string;
  - [ ] `disposition(kind)` returns the ruled import treatment (stored,
        quarantined, routed to SSH import, kept as a link), with a test per
        row of the tier architecture's §7 table;
  - [ ] the OTP display enums live here, gaining serde; castellan's `otp`
        module re-exports them from chatelaine, so there is one definition
        from P1 on, and that re-export is P1's only castellan change;
  - [ ] the production graph is serde plus `uuid` without default features:
        no `personae`, no hashing, no signing, no randomness. Proven by a
        `cargo tree` receipt and a `wasm32-unknown-unknown` check, the gaz
        and insigne precedent;
  - [ ] each kind carries exactly §1's metadata (rulings 23 and 24), no more,
        and the crate docs state the line; a test per kind constructs its
        metadata, and no kind has a field for a sealed value;
  - [ ] JSON and postcard round-trips for every kind.

- **P2 — castellan's item store on chatelaine; OTP moves.** Done when:
  - [ ] castellan holds persona-scoped sealed item records whose metadata is
        chatelaine's and whose secret payloads, one per credential, never
        appear in a chatelaine type;
  - [ ] `OtpItem` and `OtpItemId` are gone; an OTP is an item with one totp
        credential, and `OtpReleaseGate` and `OtpAdmittedSession` exercise
        that credential;
  - [ ] the `castellan/otp/v1` record formats and their legacy reader are
        removed, with no decoder (ruling 20);
  - [ ] the OTP suite passes unchanged in what it asserts: the RFC 6238 and
        4226 vectors, the release gate, the admitted session, HOTP freshness,
        Steam Guard; test edits limited to construction and imports;
  - [ ] the resident still refuses restored HOTP state and still never
        repeats a counter across independent gates, by its existing tests
        `resident_rejects_restored_hotp_state_before_releasing_it_again` and
        `independent_gates_under_one_resident_cannot_repeat_an_hotp_counter`
        (`ports/castellan/src/resident.rs`), kept.

- **P3 — the Secret Service on chatelaine (ruling 17).** Done when:
  - [ ] Secret Service collections are chatelaine collections and its items
        are items holding one `Secret` credential (content type and lookup
        attributes as metadata, the bytes sealed by castellan);
  - [ ] the D-Bus objects project from the item store; `SecretServiceStore`'s
        separate catalog/collection/item records are removed, with no decoder
        (ruling 20);
  - [ ] the resource limits (`SecretServiceLimits`) hold as before, by their
        existing tests;
  - [ ] the README's `secret-tool` store/lookup/clear receipt passes under a
        disposable session bus **on a Linux machine** (the D-Bus server is
        `cfg(target_os = "linux")`, so a Windows build proves nothing about
        it), with the machine and commit recorded;
  - [ ] **C5's chatelaine condition is met here**: chatelaine has its real
        contents and can publish once, at the Mere 0.4 baseline.

- **P4a — the agent learns RSA and ECDSA (ruling 25).** personae's SSH slots
  and agent, and castellan's SSH import, accept RSA and ECDSA keys beside
  Ed25519. Waits for Mark's review after P3 (ruling 28). Done when:
  - [ ] the agent signs with RSA using SHA-2 (`rsa-sha2-256` and
        `rsa-sha2-512`, honouring the agent protocol's signature flags) and
        with ECDSA on the curves `ssh-key` supports;
  - [ ] the RustSec advisory database is checked for every crate the new
        algorithms pull in (the `rsa` crate included), and any open advisory
        comes back to Mark before this phase lands;
  - [ ] every existing Ed25519 slot is byte-identical afterwards, and an
        Ed25519 signature from the new agent verifies exactly as before;
  - [ ] the new agent is proven against a real `sshd` while running beside
        the installed one, on its own socket or pipe; no lane replaces an
        installed agent, and where the end-to-end receipt runs is put to
        Mark at this phase's start, since it means a test key in some
        machine's `authorized_keys`.

- **P4 — CXF import (castellan feature `cxf`).** Needs P4a. Done when:
  - [ ] import is two-step (ruling 21): reading a document returns its
        accounts with counts per disposition and nothing stored; committing
        takes a persona per account;
  - [ ] every kind lands per the ruled table, proven by a fixture holding all
        17 kinds plus one unknown type, each asserted;
  - [ ] a multi-credential CXF item stays one item (ruling 16);
  - [ ] `item-reference` becomes a link; a dangling one is reported, not
        stored as an item;
  - [ ] unknown types are quarantined with their fields preserved verbatim,
        and P6's export reproduces them;
  - [ ] **quarantine is unexercisable**: a negative-control test shows the
        release gate refusing a quarantined credential, and the test fails
        if the refusal is removed (invariant 12);
  - [ ] **personae's live SSH slots are untouched** (ruling 20): `ssh-key`
        of every algorithm P4a supports goes through castellan's native SSH
        import into new fingerprint-keyed slots; a receipt shows every
        pre-existing slot byte-identical afterwards and the agent still
        serving it, and re-importing a held key is a no-op;
  - [ ] the file is treated as burning: plaintext buffers are zeroized,
        nothing is written to disk but sealed records, and the import report
        says the source file is plaintext and should be deleted;
  - [ ] all-features and nested-workspace builds pass (signalman,
        graphshell's web check).

- **P5 — the quarantine review.** Done when:
  - [ ] castellan's projection lists quarantined items as secret-free cards,
        per persona;
  - [ ] accept and delete are intents the resident authority answers, one
        item at a time; an accepted item becomes exercisable, a deleted one
        is gone;
  - [ ] the consent prompt renders on the resident's surface, as every other
        castellan approval does (invariant 1).

- **P6 — CXF export (ruling 22).** Done when:
  - [ ] a persona's items export as a CXF document behind a warning the
        caller cannot skip: the file will be plaintext, because CXP, the
        encrypted transfer, is still a 2024 working draft;
  - [ ] import, then export, then import is lossless for every kind,
        quarantined and unknown items included, by the P4 fixture;
  - [ ] exporting is an authority intent with consent, never a view.

## 4. Findings

**2026-10-01: the blast radius.** Every reference to the OTP item types, the
OTP display enums and the Secret Service store types is inside castellan:
240 occurrences in 18 files, none in graphshell, djinn or signalman
(ripgrep over `Code/repos`). P2 and P3 are castellan-internal; the compiler
is the real census.

**2026-10-01: SSH algorithms.** personae's SSH slots hold OpenSSH-encoded
private keys keyed by SHA256 fingerprint (`crates/dramatis/personae/src/ssh_slot.rs`),
and both personae and castellan build `ssh-key` with only its `ed25519`
feature. A CXF `ssh-key` credential may be RSA or ECDSA. Whether those are
quarantined, or the agent gains algorithms, is a fork for Mark at P4's
start, not a decision this plan makes. *Resolved the same day by ruling 25:
the agent gains them, as P4a.*

**2026-10-01: CXF's shape.** A Header holds Accounts ("a credential owner's
account in the exporting provider"); an Account holds Collections and Items;
a Collection lists `LinkedItem`s and nests sub-collections; an Item holds a
required array of credentials, which "are designed to be composable", and
importers "MAY" split combinations they do not support. Read from the
2026-03-09 specification.

## 5. Progress

**2026-10-01.** Plan drafted after Mark's rounds (rulings 16 to 22). Next:
P0, then P1.

**2026-10-01, later.** Rulings 23 to 29: the metadata line (§1's table), RSA
and ECDSA for the agent (P4a), and how the run goes (§6). Next: P0 and P1
lanes.

## 6. Running it

As ruled (26 to 29), with the workspace's lane rules:

- **Lanes.** P0 (Sonnet) and P1 (Opus) run together; P0 reads and reports,
  P1 writes code, and neither depends on the other. P2 and P3 follow in
  order on Opus. The run stops after P3 for Mark's review; P4a onward waits.
- **Isolation.** Each lane works in its own git worktree on its own branch,
  with its own `CARGO_TARGET_DIR` under `C:\t\cargo-targets\mere\`. Nothing a
  lane produces reaches `main` until it is verified there: the phase's
  done-conditions, its tests, the portable gate
  (`python scripts/cargo_mode.py verify`), the all-features and wasm checks
  the phase names, and every control that should fail does.
- **Forks.** A lane stops and reports any choice with more than one
  defensible answer; it does not pick. Its report becomes the next round of
  questions.
- **Records.** Lanes do not edit design documents. They report what they
  verified, with commands and counts, and this plan's §4 and §5 are written
  from the reports after verification.
- **Commits.** A lane commits on its branch, with no attribution trailer.
  Merging into `main` and pushing are done outside the lane, and pushing
  waits for Mark.
- **Linux.** P3's `secret-tool` receipt runs on the Fedora ThinkPad
  (`thinkpad-l14-f`) over SSH under a disposable session bus, with the
  commit tested and the machine recorded.
