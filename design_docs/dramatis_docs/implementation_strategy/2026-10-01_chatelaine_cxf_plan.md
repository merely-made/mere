# Chatelaine and CXF Import Plan

**Date**: 2026-10-01
**Status (2026-10-04)**: in progress. Shape ruled by Mark on 2026-10-01
(rulings 7 and 10 to 15 in the dramatis tier architecture; rulings 16 to 59
below). P0 met; P1 landed on `main` (`da3c50bc`); P2 landed (`3e4992ec`); P3
landed (`ff68e86c`), meeting the Mere 0.4 baseline's chatelaine condition.
The review stop ended 2026-10-04 (ruling 51). P4a's first build signs RSA
through `ring` and ECDSA, proven on the ThinkPad; its second round carries
rulings 54 to 58 (no re-import overwrite, P-521 refused, unsignable keys
refused, and the RSA key built once by the `rsa` crate, ring signing:
rulings 58, 59).
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
totp credential without being one. *Amended 2026-10-01*: two move whole.
castellan's `OtpKind::Hotp` carries the HOTP counter, which is mutable,
freshness-critical state and not an identifying field, so under ruling 23 it
stays sealed in castellan; chatelaine carries only the shape, named
`OtpMode` by ruling 37. *Reading, not ruled*, taken in P1's brief.

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

Rulings 30 to 33 answer P0's report (§4, 2026-10-01); rulings 34 to 38
answer P1's.

**Ruling 30.** *How exact must ruling 15's "verbatim" be?* Options:
byte-exact via raw slices; JSON-value-equal is enough; workspace-wide
`serde_json` features (`preserve_order`, `arbitrary_precision`). Mark:
**"Byte-exact via raw slices (Recommended)"**. Follows: P4 pre-scans the
document with `serde_json`'s `RawValue` and seals each unknown credential's
original bytes; `RawValue` is additive and changes nothing else in the
workspace.

**Ruling 31.** *Keep unknown fields on a known credential type?* Options:
follow the spec and ignore them; keep them sealed too. Mark: **"Follow the
spec: ignore (Recommended)"**. Follows: CXF §3.1.1's "MUST ignore unknown
fields" stands.

**Ruling 32.** *How are the crate's gaps handled?* Options: local handling
only; local handling plus drafted upstream patches; a vendored patched copy.
Mark: **"Local handling only (Recommended)"**. Follows: P4's mapping layer
re-parses the affected cases from the preserved raw JSON and reads documents
only with `from_slice`; no upstream dependency.

**Ruling 33.** *Which decoder reads CXF TOTP secrets?* Options: castellan's
own decoder; accept the crate's normalisation. Mark: **"Castellan's own
decoder (Recommended)"**. Follows: P4 decodes the raw secret string with
`ports/castellan/src/otp/base32.rs` (case, padding and spacing tolerated,
any stray character refused); a refused secret is quarantined with its
reason shown.

**Ruling 34.** *Where does an item's persona live?* Options: on the store,
checked on load; on each item. Mark: **"On the store, checked on load
(Recommended)"**. Follows: P2's item store carries one persona scope label
and refuses a mis-filed load by name, the gaz `verify_scope` precedent;
items carry no persona field. Ruling 7's "persona scope" is met by the store,
its "origin" by each item's scope.

**Ruling 35.** *What happens to an item mixing stored and quarantined
credentials?* Options: split, linked back; quarantine the whole item;
per-credential state. Mark: **"Split, linked back (Recommended)"**. Follows:
P4 moves quarantined credentials into their own quarantined item linked to
the original (CXF allows importers to split items), and routes an item's
SSH keys the same way.

**Ruling 36.** *Keep the original CXF ids?* Options: as metadata; sealed;
not at all. Mark: **"Keep as metadata (Recommended)"**. Follows: an optional
source id on items and collections, so P6 can reproduce ids and P4 can
resolve links.

**Ruling 37.** *Naming of the OTP kind and its shape.* Options: rename to
`Otp` and `OtpMode`; keep `Totp` and rename the shape only; keep both names.
Mark: **"Rename to Otp and OtpMode (Recommended)"**. Follows: the kind is
`CredentialKind::Otp` (its CXF type stays `totp`) and chatelaine's shape is
`OtpMode`; castellan's counter-bearing `OtpKind` keeps its name.

**Ruling 38.** *Accept P1's smaller readings?* (Item references live in an
item's credentials; a Secret Service secret is stored; a card's issuer is
CXF's `cardType` and a passport's document kind its `passportType`;
custom-fields keeps its section label; timestamps are optional; country
codes are uppercase; SSH fingerprint and file hash are required; `uuid` is
declared directly, because the workspace entry enables random v4 ids.)
Options: accept all; name changes. Mark: **"Accept all (Recommended)"**.

Rulings 39 to 42 answer P2's layout checkpoint (§4, 2026-10-01).

**Ruling 39.** *How are castellan's chatelaine items laid out as sealed
records?* Options: split metadata and payload records; one record per item;
a metadata book plus payload records. Mark: **"Split: metadata + payload
records (Recommended)"**. Follows: `items/<item>` (persona label and
chatelaine `Item`), `payloads/<credential>` (persona label, owning item, the
sealed payload) and a per-persona index, written payloads first, then
metadata, then index, under a per-persona transaction lock. The ruled
metadata line (23, 24) becomes the line on disk: listing, search and the
quarantine list never decrypt a secret, and an HOTP release rewrites only
its payload.

**Ruling 40.** *What does the OTP release gate address?* Options: the
credential; the item. Mark: **"The credential (Recommended)"**. Follows:
grant paths `…/otp/<persona>/<item>/<credential>`; an item-level grant still
covers its credentials (insigne's `path_covers` matches whole segments).

**Ruling 41.** *Build the per-persona item index in P2?* Options: in P2; in
P3. Mark: **"Build it in P2 (Recommended)"**.

**Ruling 42.** *Accept P2's readings?* (Every record carries the store's
persona, and a mismatch is refused naming both; an imported OTP item is
titled by its issuer, else its account; the gate refuses any item not in the
vault from P2 on; the HOTP counter leaves what hosts see; new records live
under `castellan/items/v1/<persona>/` and old `otp/v1` files are never read;
tests reading an item's account read the credential's metadata, with the
same asserted values.) Options: accept all; name changes. Mark: **"Accept
all (Recommended)"**.

**Ruling 43.** *P3's D-Bus server is Linux-only and the ThinkPad did not
answer: build now with a cross-check and run the receipt later, or wait?*
Mark: **"is the thinkpad not reachable? it's on, open, ready to go afaik"**.
Follows: he was right. It had moved from `.28` to `192.168.4.32` and does not
advertise over mDNS, so the earlier probes looked in the wrong place; `.28`
is now an iOS device (port 62078 open). `.32`'s ED25519 host key matched the
ThinkPad's recorded `SHA256:9kM6Rp…B7o`, and it is reached with
`HostKeyAlias=thinkpad-l14-f.local` rather than by editing `known_hosts`.

**Ruling 44.** *Install Rust's Linux target for cross-checks?* Options: yes;
no. Mark: **"Yes, install it (Recommended)"**. Follows: `x86_64-unknown-linux-gnu`
is installed for the pinned 1.98.1 toolchain on the Windows box.

**Ruling 45.** *Accept P2's minor choices?* (The names `OtpCredential`,
`OtpReleaseRequest.credential` and `OtpCodeTile::credential()`; `ItemStore`
public for reads and delete; `get` requires indexing; delete removes the
index entry first.) Options: accept all; name changes. Mark: **"Accept all
(Recommended)"**.

Rulings 46 to 50 answer P3's checkpoint (§5, 2026-10-01).

**Ruling 46.** *How are Secret Service collections and membership laid
out?* Options: the single index, as P2 built it; per-collection membership
records; the single index plus a resident cache now. Mark: **"Single index,
as P2 built it (Recommended)"**. Follows: membership and aliases live in the
per-persona index, one commit point, rulings 39, 41 and 45 as written; a
resident cache can come later without a disk-format change.

**Ruling 47.** *Keep or lower the default limits?* Options: keep, recording
the degradation; lower them. Mark: **"Keep, and record the degradation
(Recommended)"**.

**Ruling 48.** *How is a torn replace closed?* Options: copy-on-write; the
lock only, accepting the crash window. Mark: **"Copy-on-write
(Recommended)"**. Follows: a replace or `SetSecret` writes the new payload
under a fresh credential id, then the metadata naming it (the commit point),
then deletes the old payload.

**Ruling 49.** *Where does the `max_sessions` refusal test live?* Options:
move the session table to a portable module; a Linux-only test on the
ThinkPad. Mark: **"Move the table to a portable module (Recommended)"**.

**Ruling 50.** *Payload encoding, and P3's readings* (the item id is the
chatelaine `ItemId`, so D-Bus paths keep their form; `SecretItemId` and
`SecretCollectionId` stay newtypes; a collection's label is its title;
timestamps map to `Some(seconds)`; search covers only `Secret` credentials).
Options: keep bytes and accept the readings; base64 payloads; name changes.
Mark: **"Keep bytes; accept readings (Recommended)"**.

**Ruling 51.** *(Asked 2026-10-04 at the device pairing plan's pause, its
ruling 73: what next, with D2, chatelaine P4a or a pause as options.)* Mark:
**"Chatelaine P4a"**. Follows: the review stop after P3 (ruling 28) ends,
and P4a starts.

**Ruling 52.** *P4a teaches the agent RSA (ruling 25), and any open advisory
comes back first: RUSTSEC-2023-0071 (Marvin, CVE-2023-49092) is open with no
patch in any `rsa` release, 0.9.10 and 0.10.0-rc.18 included; the crate's
private-key operations are not constant-time, so network-observable timing
can leak the key, and its workaround says local use on a non-compromised
computer is fine. `rsa` is already in the graph through `ssh-key`, unused
for signing. How should the agent sign RSA?* Options: sign with `ring`,
parsing the key with `ssh-key`, after a feasibility check; use `rsa` and
accept the risk; ECDSA only, quarantining RSA keys. Mark: **"Sign RSA with
`ring` (Recommended)"**. Follows: P4a's lane confirms first that `ring` can
sign `rsa-sha2-256` and `rsa-sha2-512` from the key `ssh-key` parses, and
comes back if it cannot; ECDSA stays on RustCrypto's curves.

**Ruling 53.** *Where does P4a's end-to-end receipt run (a real `sshd`
accepting a login signed by the new agent, on its own pipe beside the
installed one, with a test key in that machine's `authorized_keys` for the
run)?* Options: the ThinkPad; an iMac; WSL on the laptop. Mark: **"The
ThinkPad (Recommended)"**. Follows: the test lines go into the ThinkPad's
`~/.ssh/authorized_keys` for the run and come out after, with copies before
and after compared.

**Ruling 54.** *Re-importing a held key (ruling 20's no-op) overwrites today:
castellan's import replaces the slot (`authority.rs:338`) and `ssh-add`
rewrites it at Session tier (`agent.rs:291`), silently dropping a PerUse
key's approval. What should re-import do?* Options: a no-op on both paths;
for import only; keep overwriting. Mark: **"No-op on both paths
(Recommended)"**. Follows: a held fingerprint is never rewritten, both
report "already held", and changing a held key's tier becomes its own
intent.

**Ruling 55.** *ssh-key 0.6.7 signs P-521 but its decoder refuses an ECDSA
scalar shorter than the curve width (`private/ecdsa.rs:40-53`): about 1 in 4
P-521 keys and 1 in 512 P-256 or P-384 keys, on import and `ssh-add`.*
Options: pad the scalar locally; keep P-521 and state the limit; refuse
P-521; wait for ssh-key 0.7. Mark: **"Wait, wait. Does 0.7 exist? If so, use
that now. If not, refuse p-521 until it's fixed upstream. I am not fucking
with encryption regimes and cryptography; those are waaay out of our
skillset. We should remain strictly consumers, not forking standards."**
Follows: ssh-key 0.7 has no release (crates.io lists up to `0.7.0-rc.11`,
read 2026-10-04), and `ssh-agent-lib` 0.6.0 requires ssh-key `^0.6`, so P-521
is refused until upstream fixes it. A P-256 or P-384 key the decoder refuses
is refused with a clear reason; nothing pads it.

**Ruling 56.** *Keys the agent can never sign (RSA outside ring's limits:
below 2048 or above 4096 bits, primes not multiples of 512, e below 65537;
DSA; `sk-*` security-key types) are accepted today and fail at signing.
Refuse them at the door?* Options: refuse with the reason; accept and store.
Mark: **"Refuse with the reason (Recommended)"**.

**Ruling 57.** *Adding `ring` narrows the Windows cross-check: castellan's
`keeper` and djinn no longer check for `x86_64-unknown-linux-gnu` here,
since ring's C build needs a Linux cross compiler; the `secret-service`
check still passes. How?* Options: accept and build Linux natively on the
ThinkPad; install a cross toolchain; put ring behind a feature. Mark:
**"Accept; build on the ThinkPad (Recommended)"**.

**Ruling 58.** *`ring` loads an RSA key from eight components and OpenSSH
files store six; the lane derived dP and dQ itself with crypto-bigint's
constant-time remainder. With ruling 55's principle, which?* Options: a
library derives them; keep the derivation; defer RSA. Mark: **"A library
derives them (Recommended)"**. Follows: no key arithmetic in our code; if
no maintained library builds the full key without a new crate, it comes
back to Mark. Mark's principle, refined the same day, is kept as a feedback
memory: "if the domain requires expertise due to the features being
privacy, security, or critical operation-oriented, we need to adhere to
standards rigorously ... We should rely on the wisdom of people who have
done this longer than a year."

**Ruling 59.** *Ruling 58's search found one maintained library in the lock
that builds a full RSA key from OpenSSH's six components: the `rsa` crate's
`from_components` (validate and precompute). aws-lc-rs and ring take the CRT
values as inputs (aws-lc-rs `src/rsa/key.rs:103-125`, checked), pkcs1 is a
container, and AWS-LC's no-CRT constructor is reachable only by new unsafe
FFI. The `rsa` construction runs once per key load, locally, in variable
time.* Options: `rsa` builds, ring signs; defer RSA; AWS-LC through FFI.
Mark: **"rsa builds, ring signs (Recommended)"**. Follows: the `rsa` crate's
standard construction and PKCS#8 export run once per key load; every
signature is ring's; recorded as `rsa` touching the key at load only, which
amends ruling 52's "no `rsa` private-key operation in this path" to
"no `rsa` signing or decryption".

## 3. Phases

Each phase lands with its own tests and gates and keeps the workspace green.
The nested-workspace and all-features lessons of the insigne plan (§4 there)
apply: a census or gate proves nothing about a feature or target it did not
build.

- **P0 — the CDDL diff.** `credential-exchange-format` 0.4.0 (MIT, Bitwarden,
  published 2026-06-11, unchanged since the standards survey flagged it as
  tracking the March 2025 review draft) against CXF v1.0 Proposed Standard
  with errata, 2026-03-09. Done when:
  - [x] every type and field difference is listed in §4 with its CDDL
        reference;
  - [x] each is classed: harmless, fixable by a local newtype or an upstream
        patch, or blocking;
  - [x] if anything blocks, the choice between waiting, patching upstream and
        ruling 19's alternative comes back to Mark as a fork before P4.
        Nothing blocks; the four decisions P0 raised are rulings 30 to 33.
        **P0 met 2026-10-01.**
        P1 to P3 do not depend on P0's outcome; they run after it, in order.

- **P1 — the taxonomy.** `crates/dramatis/chatelaine` gains real code.
  Done when:
  - [x] items, credentials, collections and links (`item-reference`) exist as
        plain serde types; `CredentialKind` is `#[non_exhaustive]` and covers
        CXF v1.0's 17 kinds, the Secret Service generic secret, and
        `Unknown` with its preserved type string;
  - [x] `disposition(kind)` returns the ruled import treatment (stored,
        quarantined, routed to SSH import, kept as a link), with a test per
        row of the tier architecture's §7 table;
  - [x] the OTP display enums live here, gaining serde; castellan's `otp`
        module re-exports them from chatelaine, so there is one definition
        from P1 on, and that re-export is P1's only castellan change;
  - [x] the production graph is serde plus `uuid` without default features:
        no `personae`, no hashing, no signing, no randomness. Proven by a
        `cargo tree` receipt and a `wasm32-unknown-unknown` check, the gaz
        and insigne precedent;
  - [x] each kind carries exactly §1's metadata (rulings 23 and 24), no more,
        and the crate docs state the line; a test per kind constructs its
        metadata, and no kind has a field for a sealed value;
  - [x] JSON and postcard round-trips for every kind.

- **P2 — castellan's item store on chatelaine; OTP moves.** Done when:
  - [x] castellan holds persona-scoped sealed item records whose metadata is
        chatelaine's and whose secret payloads, one per credential, never
        appear in a chatelaine type;
  - [x] the item store carries its persona scope label and refuses a load
        filed under another persona by name, with a two-persona test
        (ruling 34);
  - [x] `OtpItem` and `OtpItemId` are gone; an OTP is an item with one totp
        credential, and `OtpReleaseGate` and `OtpAdmittedSession` exercise
        that credential;
  - [x] the `castellan/otp/v1` record formats and their legacy reader are
        removed, with no decoder (ruling 20);
  - [x] the OTP suite passes unchanged in what it asserts: the RFC 6238 and
        4226 vectors, the release gate, the admitted session, HOTP freshness,
        Steam Guard; test edits limited to construction and imports;
  - [x] the resident still refuses restored HOTP state and still never
        repeats a counter across independent gates, by its existing tests
        `resident_rejects_restored_hotp_state_before_releasing_it_again` and
        `independent_gates_under_one_resident_cannot_repeat_an_hotp_counter`
        (`ports/castellan/src/resident.rs`), kept.

- **P3 — the Secret Service on chatelaine (ruling 17).** Done when:
  - [x] Secret Service collections are chatelaine collections and its items
        are items holding one `Secret` credential (content type and lookup
        attributes as metadata, the bytes sealed by castellan);
  - [x] the D-Bus objects project from the item store; `SecretServiceStore`'s
        separate catalog/collection/item records are removed, with no decoder
        (ruling 20);
  - [x] the resource limits (`SecretServiceLimits`) hold as before, by their
        existing tests; *amended 2026-10-01*: there are none (P2's finding:
        `store_tests.rs` only asserts the defaults), so P3 adds a refusal
        test per limit, each with a control;
  - [x] a replace-by-attributes cannot tear (new bytes under old metadata),
        held by the per-persona transaction lock of ruling 39; *amended
        2026-10-01*: the lock alone does not cover a crash between the
        payload and metadata writes, so replace and `SetSecret` are
        copy-on-write (ruling 48), proven by crash-point tests;
  - [x] the `max_sessions` refusal test runs on every platform, its session
        table moved out of the Linux-only D-Bus module (ruling 49);
  - [x] the secret bytes handed to zbus (`dbus/objects.rs:369`,
        `dbus/service.rs:192`, plain `to_vec()` copies today) are zeroized on
        our side, with what remains outside our reach stated;
  - [x] the README's `secret-tool` store/lookup/clear receipt passes under a
        disposable session bus **on a Linux machine** (the D-Bus server is
        `cfg(target_os = "linux")`, so a Windows build proves nothing about
        it), with the machine and commit recorded;
  - [x] **C5's chatelaine condition is met here**: chatelaine has its real
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
  - [ ] a multi-credential CXF item stays one item (ruling 16), unless its
        credentials' treatments differ: then the quarantined credentials
        move to their own quarantined item linked back to the original, and
        its SSH keys route to SSH import (ruling 35);
  - [ ] documents are read whole with `from_slice` only, never a reader or a
        `Value`, since the crate's base64url strings deserialize only when
        borrowed; each credential's `type` is checked to be a string before
        the typed parse (ruling 32);
  - [ ] a known type that fails its typed parse, which the crate demotes to
        `Unknown` with the known type string, is re-parsed locally from the
        raw JSON (TOTP `period` and `digits` up to 65,535; custom-fields
        element by element) and, if that still fails, quarantined with its
        reason, never treated as an unknown type (ruling 32);
  - [ ] unknown credentials are sealed as their original bytes, taken from a
        `RawValue` pre-scan (ruling 30); unknown fields on known types are
        ignored, as CXF §3.1.1 requires (ruling 31);
  - [ ] TOTP secrets are decoded from the raw string by castellan's own
        Base32 decoder, never the crate's normalising `B32`; a refused
        secret is quarantined with the reason shown (ruling 33);
  - [ ] the spec's ignore rules are applied in the mapping layer: an unknown
        TOTP algorithm, an unknown HMAC algorithm, and sharing accessors with
        an unknown type or permission; and a document whose major version is
        not 1 is refused;
  - [ ] no crate type is ever logged or formatted with `{:?}`: the crate
        derives `Debug` on secrets and has no zeroize;
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

**2026-10-01: P0, the CDDL diff (Sonnet lane; claims re-checked in the crate
source).** `credential-exchange-format` 0.4.0's README still claims the
March 2025 review draft (`README.md:5-7`), but its wire shapes match the
2026-03-09 Proposed Standard: all 17 types, every enum, the Shared extension,
and Appendix A, which parses and re-serializes JSON-equal. MIT; dependencies
`chrono` (no clock), `data-encoding`, `serde`, `serde_json`; no `unsafe`;
compiles for `wasm32-unknown-unknown`. Unknown types arrive as
`Credential::Unknown { ty, content }` (`src/lib.rs:187-193`, an untagged
fallback), with raw JSON kept value-equal, not byte-equal. The differences
that can bite a conforming document, all fixable locally and none losing
data:

- TOTP `period` and `digits` are `u8` (`src/login.rs:108,112`) where the spec
  allows 16 bits, so larger values fall to `Unknown { ty: "totp" }`;
- `B32` uppercases a TOTP secret and then *drops* every character outside the
  alphabet (`src/b64url.rs:128-129`), so a mangled secret can decode to a
  different valid one;
- one unknown field type or bad value demotes a whole custom-fields
  credential to `Unknown`;
- base64url strings deserialize only borrowed (`try_from = "&str"`,
  `src/b64url.rs:6,81`), so `from_reader` and `from_value` fail;
- a known type whose typed parse fails is demoted to `Unknown` with the
  known type string, so `Unknown` does not mean "unrecognised";
- unknown fields on known types are dropped, as CXF §3.1.1 requires;
- `Debug` is derived on everything, secrets included, and there is no
  zeroize.

The spec's own Appendix A covers 15 of the 17 types (custom-fields and
item-reference are missing), and §3.4.2 contradicts itself on unknown field
types. Rulings 30 to 33 settle the four decisions P0 raised.

**2026-10-01: P1, the taxonomy (Opus lane).** Built as `71a91267` on its
lane branch; verified by a merge onto `main` in a worktree at normal depth
(`f8734195`): chatelaine 51 tests, castellan 90 with every feature, the
production tree serde and `uuid` only (no `v4`, `std` or getrandom),
wasm32, clippy, and the portable gate. The lane's own gate run failed only
because graphshell's `practice_disclosure.rs:220` and
`practice_workspace.rs:591` `include_str!` a woodshed file by a relative path
that assumes `Code/repos/mere`, which a deeper worktree misses; any clone not
beside woodshed fails the same way. Its control broke one disposition and one
validation, and three named tests failed. Castellan's change was the enum
re-export and the dependency line only. Findings carried to later phases:
castellan now depends on chatelaine, so chatelaine must publish before
castellan's next publish (C5's baseline does both); CXF ids are opaque
strings, not UUIDs (ruling 36); a CXF `LinkedItem` may point into another
account, which ruling 21 may map to another persona, which is a P4 fork;
CXF totp's `username` is optional while castellan's OTP account is required,
and CXF digits can fall outside castellan's 6 to 10, both P4 forks.

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

**2026-10-01, P0 and P1.** P0 met (§4); Mark ruled its four decisions as 30
to 33. P1 built and verified green (§4); Mark ruled its forks and readings
as 34 to 38, and the lane is making the two type changes they require
(source ids, the `Otp` and `OtpMode` names) before P1 merges. The Fedora
ThinkPad did not answer mDNS in three rounds while both iMacs did, so P3's
receipt needs it woken first.

**2026-10-01, P1 landed.** The follow-up pass (`f0141e3a`) added `SourceId`
(unpadded url-safe base64url, 1 to 64 bytes, kept verbatim) on items and
collections, and the `Otp` and `OtpMode` names; its control (accepting 65
bytes) failed two named tests. Two consequences of ruling 37 the lane
applied, *reading, not ruled*: chatelaine's own serialized tag for the kind
is `otp` (its CXF type stays `totp`), and the field holding the mode is
`mode`. Merged onto `main` in a normal-depth worktree and verified there
(`da3c50bc`): chatelaine 53 tests, castellan 90 with every feature, the tree
(serde and `uuid` only), wasm32, clippy, and the portable gate, all exit 0;
the three "patch was not used" warnings are lock-wide and predate P1 (the
root `Cargo.toml` is unchanged). *Amended 2026-10-02*: predating P1 did not make them
harmless. The `iroh-mdns-address-lookup` one means H10's per-interface
mDNS fix is not in effect; see the device pairing by key plan's §6. The chatelaine README now says the crate
holds identifying metadata and that persona scope lives on castellan's
store. `main` fast-forwarded; nothing pushed. Next: P2.

**2026-10-01, P2 checkpoint.** The P2 lane stopped before writing the store,
as briefed: personae's sealed records are one path each, the path bound into
the encryption (`sealed_record_storage.rs:85`), with no listing and no
cross-record transaction (`update_record` is single-path), and the Secret
Service's item records hold their secret bytes (`StoredItem.secret`), so
every property read and search decrypts the secret. Its measurements:
personae's envelope writes plaintext and ciphertext as JSON number arrays,
so a sealed file is 12.7 to 23 times what it seals (a 1 MiB secret makes a
~13.4 MB file), worth knowing for the personae tier, outside this plan.
Mark ruled its forks as 39 to 42; the lane is implementing.

**2026-10-01, P2 landed.** Built as `23e2b43b` (lane, Opus) and merged onto
`main` as `3e4992ec` after verification in the normal-depth worktree.
`castellan::items::ItemStore` keeps one persona's items under
`castellan/items/v1/<persona>/` as `items/<item>`, `payloads/<credential>`
and `index` records (ruling 39), each carrying the persona label; an item is
visible only once indexed; inserts write payloads, metadata, index under the
per-persona transaction lock, now shared with the Secret Service store;
`Payload` has no `Debug` and zeroizes on drop; `list` and `get` never open a
payload; `exercise` refuses a non-vault item and advances an HOTP counter
through one `update_record` on the payload. OTP is items holding one `Otp`
credential, exposed to hosts as `OtpCredential` (item plus credential id);
the gate is credential-addressed (ruling 40); `castellan/otp/v1` and its
legacy reader are gone with no decoder (ruling 20). Verified:

- castellan 89 unit tests with every feature (82 before, 7 new), 3 + 4
  integration and the doctest; 66 with default features; chatelaine 53;
- castellan's test names on `main` and merged compared by listing: nothing
  removed, the 7 additions only (the doctest moved from line 24 to 27);
- clippy 122 warnings, the same as before, none in a touched file;
- signalman's separate workspace checks; the portable gate passes;
- the lane's four controls each failed only their target: the persona check
  made always-true, the freshness ledger bypassed, the index written before
  the payloads, and metadata reads opening payloads.

The lane's choices, *reading, not ruled*: the names `OtpCredential`,
`OtpReleaseRequest.credential` and `OtpCodeTile::credential()`; `ItemStore`
public for reads and delete, with `insert` and `exercise` crate-private;
`get` also requires the item to be indexed; a delete removes the index entry,
then payloads, then metadata, so an interrupted delete leaves only an
invisible orphan. Carried to P3: `get` and `exercise` load the whole index on
every call, which at `SecretServiceLimits` scale (32 collections of 4,096
items, an index of about 5 MB) a D-Bus property read would repeat.

**2026-10-01, the Linux receipt proven before P3.** On the ThinkPad
(`thinkpad-l14-f`, Fedora 44, now at `192.168.4.32` and recorded in
`known_hosts` by matching fingerprints, with Mark's go-ahead), `main` at
`f7b31b9a` arrived as a git bundle into a separate worktree,
`~/Code/repos/mere-receipt`, beside the machine's own checkout, which stayed
on its branch and clean. Under 1.98.1 (installed there by rustup),
`dbus-run-session -- cargo test -p castellan --features secret-service
--test secret_service_linux --locked -- --ignored` passed
`secret_tool_store_lookup_and_clear` (1 test). This is P3's positive control:
the receipt works on that machine before P3 changes anything.

**2026-10-01, P3 checkpoint.** The P3 lane (Opus) stopped before
collections, as briefed, and landed only layout-independent work: a refusal
test per store-enforced limit on the pre-P3 store, each with a control that
failed only its own test (`e7c8acbd`, lane branch). Its measurements
(release build, resident storage with the freshness ledger, this laptop): at
the limits (32 × 4,096 = 131,072 items), an index holding ids and membership
is 11.4 MB plaintext and 40.7 MB on disk, 534 ms to load and 1,233 ms to
save; ids alone are 5.1 MB and 18.3 MB (so P2's "about 5 MB" counted ids
only); a realistic 200-item index is 17.7 KB, about 1 ms to load; one item
metadata record loads in 0.27 ms, so a metadata-only search at the limits is
about 39 s under any uncached layout; every save costs at least ~28 ms
(three fsyncs). Today's store makes a 1 MiB secret's item record 15 MB on
disk and loads it on every D-Bus property read (95 ms). Mark ruled its
forks as 46 to 50, keeping the defaults with this degradation recorded; the
lane is implementing.

**2026-10-02, P3 landed.** Built as `e7c8acbd` (the limit tests) and
`0891771f` (lane, Opus); merged onto `main` as `ff68e86c` after verification
on Windows in the normal-depth worktree and on the ThinkPad. The Secret
Service now runs on chatelaine items: `SecretServiceStore` is `{ items:
ItemStore, limits }`; collections and aliases live in the per-persona index
(ruling 46); an item is a chatelaine item holding one `Secret` credential;
metadata reads and search never open a payload, and only `secret()` does,
through `exercise`; replace and `SetSecret` are copy-on-write under a held
per-persona `Transaction` (ruling 48); the old `castellan/secret-service/v1`
records and code are gone with no decoder (ruling 20); the session table is
portable (ruling 49); public signatures and the D-Bus surface are unchanged,
the `Secret` struct still `(oayays)`. Inbound and outbound secret bytes are
held in a zeroizing `SecretBytes` and moved without copies; what remains
outside castellan's reach is zbus's own message buffers, kernel and bus
buffers, the client process, and heap fragments freed when serde grows a
buffer (personae's JSON payload load and save do this for OTP too, a
personae-tier finding). Verified:

- Windows: castellan 102 unit tests with every feature (97 before; nothing
  removed, 13 added: 6 limit refusals, 5 replace and metadata-only, 2
  sessions), 3 + 4 integration and the doctest; 66 with default features;
  chatelaine 53; clippy 122, unchanged; signalman; the portable gate; the
  Linux library cross-check;
- **the ThinkPad**, natively, every feature, under a disposable session bus
  with ignored tests included: 102 unit tests, among them the first build and
  pass of the Linux-only `secret_bytes_keep_the_secret_struct_signature`;
  3 + 4 integration; `secret_tool_store_lookup_and_clear`, the receipt that
  passed on `main` before P3; the doctest; the machine's own checkout left
  on its branch and clean;
- the lane's nine controls each failed only their target: an in-place
  payload write (two replace tests), search opening payloads, and each of
  the seven limit checks removed.

Measured at the limits (ruling 47; Windows 11 laptop, release build, resident
storage with the freshness ledger, medians of 3 to 5 runs except single runs
for create, delete and search; the laptop varied about 2.5× between days, so
these are ranges): an index load 208 to 534 ms; a property read 199 to 211 ms
(95 to 113 ms per read before P3 for a 1 MiB secret, which it decrypted);
`items(collection)` 0.9 to 1.2 s; `secret()` 203 to 282 ms; `set_secret` 235
to 333 ms; `create_item` 0.9 to 1.4 s; `delete_item` 0.9 to 1.2 s; a search
of 131,072 metadata records 19.8 to 23.7 s. At a realistic 200 items an index
loads in 0.55 to 0.88 ms. Over D-Bus, `CreateItem` with `replace` first runs
a full search to choose its signal, a pre-P3 behaviour kept as is.

The lane's readings, *reading, not ruled*: a collection's non-`Secret`
members are skipped by `items()` and `search` and counted by the
per-collection limit (none can exist before P4); `collections()` lists
top-level collections only; item-store refusals surface as a new
`SecretServiceError::Items` variant, mapped to D-Bus `Failed`; a replace
changes the credential id, which the Secret Service never exposes.

**The run stops here for Mark's review** (ruling 28). P4a, the agent's RSA
and ECDSA, waits.

**2026-10-04.** The review stop ended (ruling 51), and P4a starts with
rulings 52 (RSA signs through `ring`, after a feasibility check, since
RUSTSEC-2023-0071 is unpatched in every `rsa` release) and 53 (the
end-to-end receipt on the ThinkPad).

**2026-10-04: P4a, first build** (Opus lane, `2d2a36c6`, on `d0d8372b`; not
merged).

- **ring is feasible.** `rsa::KeyPair::from_components` (ring 0.17.14
  `src/rsa/keypair.rs:219`) takes n, e, d, p, q, dP, dQ and qInv, and
  OpenSSH's `iqmp` is ring's qInv. n must be 2048 to 4096 bits with each
  prime a multiple of 512 bits, and e at least 65537. Its private
  exponentiation is constant-time, and it has no public SHA-1 signing, so a
  flagless `ssh-rsa` request is refused (*reading*). dP and dQ were derived
  by the lane, which ruling 58 replaces.
- **The design.** `personae/src/ssh_sign.rs` routes by key type: Ed25519
  and ECDSA through ssh-key's RustCrypto signers, RSA through ring, the
  request's flags choosing SHA-256 or SHA-512 (SHA-256 when both are set,
  as OpenSSH does). The slot format is unchanged, and the lock gains two
  dependency lines with no new packages.
- **Evidence:**
  - 11 signing tests verify with ssh-key's public verification, under
    seven controls.
  - An existing Ed25519 slot is byte-identical and signs as before.
  - castellan imports RSA and ECDSA into new fingerprint-keyed slots beside
    an untouched one.
  - On the ThinkPad's `sshd` (OpenSSH 10.2p1), logins with RSA-SHA2-256,
    RSA-SHA2-512 and ECDSA P-256, P-384 and P-521 all succeeded through a
    receipt agent on its own pipe, and were refused before and after.
    `authorized_keys` came back byte-identical (sha256 `bc629d2e…f4bb`).
  - Mark's own SSH worked throughout, and PID 53336 was untouched.
- **RustSec** (advisory-db `ef6173cb`): only RUSTSEC-2023-0071 on `rsa` is
  open, and no `rsa` private-key operation is in the signing path. ring,
  untrusted, spin and the rest are patched or clear.
- **Found:**
  - ssh-key 0.6.7's conversion to the `rsa` crate passes p twice instead
    of p and q (`private/rsa.rs:199-203`, checked), so the agent before P4a
    could not sign any real RSA key.
  - Its ECDSA decoder refuses short scalars (ruling 55).
  - `ssh-add` of a held key overwrote it at Session tier (ruling 54).
- **Gates:** personae 166 and castellan 104 unit tests, djinn and the linux
  `secret-service` check pass. The portable gate fails only on the known
  worktree-depth `include_str!` errors. Clippy is unchanged.

## 6. Running it

As ruled (26 to 29), with the workspace's lane rules:

- **Lanes.** P0 (Sonnet) and P1 (Opus) run together; P0 reads and reports,
  P1 writes code, and neither depends on the other. P2 and P3 follow in
  order on Opus. The run stops after P3 for Mark's review; P4a onward waits.
- **Base.** A lane first confirms its worktree is based on `main`'s tip:
  P2's was created at an older commit, before P1 and rulings 30 to 38, and
  its lane caught it by fast-forwarding.
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
  commit tested and the machine recorded. The commit travels as a git
  bundle, so an unverified lane branch is never pushed; the ThinkPad's own
  checkout is restored to its branch afterwards. Lanes iterate on
  Linux-only code with `cargo check --target x86_64-unknown-linux-gnu`
  (ruling 44), which is evidence about compiling only, never a receipt.
  Proven 2026-10-01: `cargo check -p castellan --features secret-service
  --lib --target x86_64-unknown-linux-gnu` exits 0, and its dep-info lists
  all four `secret_service/dbus/*.rs` files, which a Windows build never
  compiles. `--tests` and `--all-features` fail on `ring`'s C build script,
  reached only through castellan's dev-dependency on gazette (`reqwest`,
  `rustls`), so tests are compiled on the ThinkPad.
