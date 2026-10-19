# Standards Across the Stack: What to Design To

**Date:** 2026-08-24
**Kind:** research brief (survey; nothing here is scheduled)
**Anchors:** [credential port + gazette brief](mere_docs/research/2026-08-10_credential_port_gazette_brief.md)
(corrected in place by this pass), [W3C standards architecture review](mere_docs/research/2026-07-05_w3c_standards_architecture_review.md)
(owns the web-platform spine; this brief deliberately does not re-cover it),
[turnstone suite composition census](2026-08-22_turnstone_suite_composition_and_capability_census.md),
[crypto stack decision](mere_docs/technical_architecture/2026-08-10_crypto_stack_decision.md).

**Scope note.** This is the workspace's one home for cross-repo standards
findings and the reasoning behind them, per DOC_POLICY §2: genet, retinue,
smolweb, woodshed and the rest cite it by path
(`mere/design_docs/2026-08-24_standards_survey_brief.md`) rather than keeping
copies. Since 2026-10-18 the verdicts themselves live in the family spec
registry, `merelyllc.com/content/specs.toml` (see §7). Plans that act on any finding here live in their own repo and
point back at the relevant section.

The origin question (Mark, 2026-08-24): are there password-vault standards we
should adhere to, and standards relevant to the broader stack — with the
observation that genet is bound to W3C/WPT by contingency, while fleece, pelt and
tabard might have standards we could *design to* rather than merely comply with.
That distinction organises the whole brief.

Verdict grammar is the one the [W3C review](mere_docs/research/2026-07-05_w3c_standards_architecture_review.md)
established: **ADOPT** (build to it, it is load-bearing), **PULL** (design for it
now, implement in order: the named consumer's data model reserves room for it
today, and the implementation lands in its turn), **SKIP** (deliberately not,
on the record), **WATCH** (moving target, re-check). **PULL was redefined by
Mark on 2026-09-23**; it first meant "implement when a real consumer demands
it". §7.1 re-reads every PULL row under the new meaning.

---

## 1. What went stale, and what that says about the shelf life of a standards doc

Three findings invalidated statements in the 2026-08-10 credential brief, all
corrected there in place this session. They are recorded here because the
*pattern* matters more than the three facts.

- **The ssh-agent protocol is now RFC 9987** (Standards Track, May 2026, IETF
  SSHM WG), superseding the expired individual `draft-miller-ssh-agent`.
  personae has shipped an agent since 2026-07-22; it can now make a conformance
  claim. The constraint-extension namespace is normatively specified, which
  makes it the sanctioned wire slot for castellan's per-persona release policy
  rather than a side channel. The `sk-*` FIDO key types remain outside the RFC
  as `@openssh.com` vendor extensions, so conformance says nothing about
  hardware-backed keys — precisely the half a credential keeper cares about.
- **CXF v1.0 is a Proposed Standard**, approved August 2025 with errata folded
  in 2026-03-09. The 2026-08-10 brief called it "drafts public"; castellan's
  README still calls CXF import "follow-on work" against a review draft that is
  a year behind. Note the trap that produced the stale reading:
  fidoalliance.org's surrounding prose still says "early review draft" while the
  artefact links resolve to `cxf-v1.0-ps-errata-20260309.html`. Trust the
  artefact.
- **Linux has an emerging third-party passkey provider seam.** `credentialsd`
  plus a proposed XDG credential portal (linux-credentials org, with
  `libwebauthn` and `oo7`) replaces the old "no blessed seam, use virtual
  hidraw" position.

The lesson for this repo: a standards inventory decays in months, not years, and
it decays *silently* — every stale line still reads as authoritative. Anything
in this brief with a status attached carries a verification date for that reason.

---

## 2. Castellan: the vault

The primitives are right and this brief does not propose changing them.
`personae` seals with Argon2id (RFC 9106) and ChaCha20-Poly1305 /
XChaCha20-Poly1305 (RFC 8439), signs with Ed25519, uses a 16-byte salt and a
32-byte key — which is exactly what RFC 9106 §4 asks for. The
[crypto stack decision](mere_docs/technical_architecture/2026-08-10_crypto_stack_decision.md)
already rules the RustCrypto+dalek posture and already holds `argon2` at 0.5.3 on
the correct grounds that a password-hash change is a stored-format change.

Every real gap is in **crypto agility**, not in crypto choice.

### 2.1 Neither derivation root records its own parameters

Verified in code, 2026-08-24:

- `ports/djinn/castellan/src/custody/passphrase_storage.rs` — the on-disk
  `EncryptedFile` carries `version`, `salt`, `profiles`. It does **not** carry
  the Argon2 cost parameters; `grep` for `m_cost`/`t_cost`/`p_cost` across both
  `passphrase_storage.rs` and `passphrase_root.rs` returns nothing. The module
  doc at the head of `passphrase_storage.rs` nevertheless describes the wire
  format as "ciphertext + nonce + Argon2id parameters". The doc is wrong about
  its own format.
- `derive_kek` calls `Argon2::default()`, so the parameters are whatever the
  compiled-in crate says: argon2 0.5.3 defaults to m=19456 KiB, t=2, p=1. That
  is the OWASP low-memory baseline, and it is below *both* of RFC 9106 §4's
  recommended options (first: t=1, p=4, m=2 GiB; second: t=3, p=4, m=64 MiB).
- `repos/dramatis/crates/personae/src/keypair.rs` — `derive_child` derives every
  persona key as `Ed25519::from_seed(blake3::keyed_hash(master_key, salt))`. A
  sound PRF, but with no versioned context slot.

Two consequences, both structural. A future `argon2` release that changes
`DEFAULT_M_COST` silently makes every existing vault file underivable, with no
version field to trigger a migration. And the cost parameters can never be
raised, because raising them is indistinguishable from corruption. RFC 9106 and
the PHC string format exist partly to prevent exactly this; BLAKE3's own
`derive_key` context-string mode, or RFC 5869 HKDF's `info` parameter, is the
equivalent fix on the derivation side.

> **Revised 2026-08-24, after a code-level pass.** The paragraph that stood here
> called this "a v2-root decision rather than a patch" and argued for taking it
> before the vault carries real credentials. Both halves were wrong, and the
> corrected version is cheaper:
>
> - **The passphrase ladder holds no data.** There is no `vault.json` and no
>   `vault-root.pass.json` anywhere on the machine. The live vault is the
>   DPAPI/auto-unlock ladder (`AppData\Local\personae\vault\`, plus a second
>   store under `Hocket\personae\`). The Argon2 question touches zero bytes of
>   real data today.
> - **The SSH key is not at risk from either change.** It is a Direct slot with
>   lineage `LocallyGeneratedExternallyRegistered` (`ssh_slot.rs:53-59`) holding
>   OpenSSH private-key bytes — stored, not derived — and it lives on the DPAPI
>   ladder. `derive_child` cannot orphan it.
> - **The two problems have nearly disjoint blast radii.** `derive_kek` has 3
>   in-crate callers. `derive_child`'s public surface has ~118 call sites across
>   73 files, with `DerivedKeyAttestation` on the wire in 21 of them and a hard
>   version gate at `provider.rs:44`.
> - **Read-old/write-new is already house style here.**
>   `sealed_record_storage.rs:30-31` carries `LEGACY=1` and `CURRENT=2`
>   simultaneously, accepts both, and always writes 2. So the format change needs
>   no flag day and no migration to write.
>
> Corrected sequencing: **the format and parameter change is patch-shaped and can
> land alone.** The derivation-context change is the expensive one — it requires
> every peer to ship a v2-aware verifier before any peer mints a v2 key, and the
> SSH CA (`ssh_ca.rs:139-141`, enrolled as a `cert-authority` line on the Linux
> laptop's sshd) needs manual re-enrollment regardless. The two are independent;
> bundling them holds a cheap fix hostage to an expensive one.
>
> One further gap: the `argon2` hold recorded in the crypto stack decision is
> **not enforced by the manifest**. `repos/dramatis/crates/personae/Cargo.toml:44`
> reads `argon2 = "0.5"` (caret); only `Cargo.lock` pins 0.5.3. Since 0.6 changes
> the default parameters, a `cargo update` is exactly the silent-unlock-failure
> path this section describes.
>
> Note also that `p=4` buys no wall-clock speedup here: argon2 0.5.3 has no
> threading and its lane loop is serial, so parallelism is an anti-GPU structural
> parameter only. Work scales as `m x t`, making RFC 9106's second recommended
> option ~5.1x current cost and the first ~53.9x. The first option is
> *unallocatable* on wasm, so it is not coherent across mere's declared targets
> even in principle.

The `argon2` 0.5.3 hold already recorded in the crypto stack decision is the
right instinct; this is the same instinct applied to the file format.

**Verdict: ADOPT RFC 9106 §4 as a named parameter target and record it in the
format; ADOPT RFC 5869 (or BLAKE3 `derive_key`) as the versioned derivation
context.** The first is small and cheap today. The second is not, and should be
sequenced on its own terms.

### 2.2 `otpauth://` has no normative specification at all

Castellan already imports it. There is no RFC, no consortium spec: the de-facto
authority is a Google wiki page, `otpauth` sits at IANA only as a *provisional*
URI scheme, and two competing individual drafts (`draft-andesco-otpauth-uri`,
`draft-linuxgemini-otpauth-uri`) disagree on issuer semantics. Implementations
diverge on Base32 padding and on the SHA-256/SHA-512 algorithm values.

The [OTP plan](archive_docs/2026-10-06_completed_plans/2026-08-10_castellan_otp_plan.md)
already refuses `otpauth-migration://` on exactly these grounds. The plain scheme
deserves the same note on the record: RFC 6238 and RFC 4226 are Informational
RFCs with published test vectors, but the URI that actually carries the seed
between vaults is unspecified, and castellan is already interoperating with it.

**Verdict: no change to behaviour, but record the negative.** This is what CXF is
for — CXF v1.0 defines a dedicated TOTP credential type with actual normative
text.

### 2.3 CXF is the runway item, and it is plaintext by design

One parser buys import from 1Password, Bitwarden, Dashlane, Apple Passwords,
Chrome and Android. The mapping is not a hack: CXF `BasicAuth` maps to chatelaine
passwords, `TOTP` onto the existing RFC 6238 items, `SSHKey` onto personae's
`ssh_slot`, `Passkey` is the forward hook, and the
Header/Account/Collection/Item/Credential tree gives face-scoped separation a
natural landing spot on import. Export in the same format is the anti-lock-in
claim the product needs to make credibly — a local-first vault that cannot get
your data back out is a worse cloud.

Two gotchas, both material:

1. **CXP has not moved since 2024-10-03** and is still a Working Draft. It was
   the half that carried the encryption. CXF explicitly "does not make any
   assumptions about the protocol used for the transfer", so a `.cxf` file on
   disk is plaintext credentials. Every import path must treat the file as
   burning; any export either waits for CXP or gets wrapped by castellan itself
   with a loud, unavoidable warning.
2. **The Rust crate lags the spec by a year.** `credential-exchange-format`
   0.4.0 (MIT, Bitwarden) still declares the March 2025 review draft. Adopting
   it without diffing against the 2026-03-09 CDDL silently inherits a stale
   schema.

The real cost is policy, not parsing: 17 credential types include `Passport`,
`DriversLicense` and `CreditCard`, and which of those castellan stores, drops, or
quarantines on import is a decision for Mark, not an implementation detail.

**Verdict: ADOPT CXF (import first). WATCH CXP.**

### 2.4 A WebAuthn credential is scoped to an RP ID, not to a persona

The identity model partitions by cryptographic face. WebAuthn has no
representation for "which face is this passkey" — the credential is bound to a
relying-party ID. That collision is architectural, it is not solvable inside the
standard, and it is better on the record now than discovered when someone tries
to use two personae on the same site. WebAuthn L3 is a **Candidate
Recommendation Snapshot (26 May 2026)**, proposed for advancement 20 July 2026;
it is routinely cited as a Recommendation and is not one.

### 2.5 The rollback gap has a standard that closes it

Castellan's README is honest that `sealed_record_storage/freshness.rs` detects
rollback only when its separately-rooted ledger was not restored alongside the
record store. **TPM 2.0 NV monotonic counters** (TCG TPM 2.0 Library
Specification, published as ISO/IEC 11889) are the standard answer. Windows and
Linux only — macOS Secure Enclave counters have no public API.

Revision caveat: the survey cited rev 01.83 and the fact-check found 1.85
(released 2026-03-12), but could not read the TCG listing page, which returns 403
to automated fetch. **Re-pin the revision by hand before this goes into a plan.**
The load-bearing content — NV counters — is unchanged across revisions.

**Verdict: PULL**, with a real consumer (the freshness ledger) already present.

### 2.6 Refused, on the record

- **Post-quantum for the vault is a trap.** Sealed records are symmetric and are
  nobody's harvest target. The genuine harvest-now-decrypt-later exposure is the
  X25519 in the P2P *sync transport* — a different artifact. Treating "PQ the
  vault" as the task spends a year on the wrong thing. FIPS 203/204/205: WATCH,
  scoped to transport.
- **FIPS 140-3, SOC 2, ISO/IEC 27001, PCI DSS, MASVS: SKIP.** All are
  organisational or tenancy regimes. A serverless local-first product with no
  tenants is not the subject of any of them. OWASP ASVS 5.0.0 is the one worth a
  PULL, as a checklist rather than a certification.
- **MLS (RFC 9420): SKIP for the vault.** Too much group-state machinery for what
  shared credentials need. **RFC 9180 HPKE is the smaller right thing** — the
  sealing primitive without the group state machine, already the substrate under
  both MLS and TLS Encrypted ClientHello, with pure-Rust implementations. It is
  also what murmur's store-and-forward actually wants: seal a payload to a
  recipient's public key offline, no handshake, no online counterparty.
- **OPAQUE (now RFC 9807, no longer a draft) and SRP: SKIP** — there is no sync
  server to authenticate against. Note RFC 5054 is Updated by RFC 8996, which
  struck the TLS versions SRP ran on.

### 2.7 Foreign key material arrives in formats nobody chose

chatelaine's remit explicitly includes "foreign key material", and `import`
migrates stored browser data. Two omissions the fact-checkers flagged:

- **RFC 9580 (OpenPGP, July 2024)** obsoletes RFC 4880/5581/6637 and is the first
  OpenPGP RFC to standardise v6 keys, AEAD-protected packets and Argon2 S2K.
  Governance gotcha worth deciding deliberately: GnuPG declined 9580 in favour of
  LibrePGP, so "OpenPGP compatibility" is now two mutually incompatible things.
  Rust's `sequoia-openpgp` and `rpgp` sit on the 9580 side.
- **RFC 7292 (PKCS #12)** with RFC 5958 — the canonical trap of this lane. An
  ASN.1 container whose legacy MAC/encryption profile modern toolchains reject by
  default. Note **RFC 9879 (2025)** obsoletes RFC 9579 and updates both RFC 7292
  and RFC 8018; it is the current normative text for a `.p12` carrying a modern
  PBKDF2/HMAC-SHA-256 integrity MAC, and it changes the password encoding from
  BMPString to UTF8String.

### 2.8 Secret Service: the most valuable OS surface, with a silent failure mode

Castellan implements the server side of Secret Service 0.2 on zbus 5.19. That is
the only OS surface in the stack where castellan can *be* the service rather than
plug into someone else's: every libsecret consumer on Linux — GNOME apps,
Chromium's password store, NetworkManager, the Git credential helper, Evolution —
becomes a client without knowing castellan exists. Nothing on Windows or macOS
offers this.

Status, verified 2026-08-24: still **version 0.2, still labelled DRAFT**, with a
publication date of 2026-04-08 (the document is being rebuilt; the version did
not move). The canonical URL moved —
`specifications.freedesktop.org/secret-service-spec/latest/` now 301s to
`.../secret-service/latest/`.

The operational finding, which is the one that matters: **only one process can
own `org.freedesktop.secrets` on a session bus.** Castellan's
own-without-replacement posture is correct, but on a stock GNOME session
gnome-keyring wins and castellan is **silently inert**. Per the house rule that
diagnostics assert invariants rather than merely measure cost, this wants a
runtime assertion that reports which process holds the name — not a comment.

Second finding: the `dh-ietf1024-sha256-aes128-cbc-pkcs7` session algorithm is
weak by modern standards. The honest position, which castellan's design already
assumes, is that Secret Service transfer security comes from the bus and the peer
check (caller binding, `/proc` executable identity), not from the session
algorithm.

---

## 3. genet: fleece, tabard, pelt

Mark's instinct was that these three might carry standards to design to rather
than comply with. Confirmed for all three, most strongly for fleece.

### 3.1 fleece has the best design-to story in the stack

The **Web Annotation Data Model** has been a stable W3C Recommendation since
2017-02-23 and defines eight selector types — `FragmentSelector`, `CssSelector`,
`XPathSelector`, `TextQuoteSelector`, `TextPositionSelector`,
`DataPositionSelector`, `SvgSelector`, `RangeSelector` — plus a `refinedBy`
composition rule. A `TextPositionSelector` refined by a `TextQuoteSelector` is a
fast path and a robust fallback in one object.

fleece is render-free and already walks a profile-neutral LayoutDom in document
order. It can therefore mint exact/prefix/suffix quotes and codepoint offsets
essentially for free *while it is already traversing*. Emitting standard
selectors makes every downstream consumer — gazette's feed pipeline, alembic's
recall, knot — composable with the rest of the world instead of with a
house-private anchor format. Rust prior art is effectively absent: crates.io has
no general implementation.

Pair it with **URL Fragment Text Directives** (`#:~:text=prefix-,exact,-suffix`),
the same idea as a URL a person can paste, shipped in Chrome 80+, Edge 83+,
Safari 16.1+ and Firefox 131+. For a serverless house this is provenance with
zero infrastructure. Three caveats: it is still a WICG Draft Community Group
Report (upstreaming to WHATWG is open as `whatwg/html` PR #11895), so pin the
commit; the `:~:` directive is deliberately **stripped from `document.URL`** as
an anti-exfiltration measure, and genet must reproduce that stripping or it
becomes a privacy leak; and Chrome gates activation on user-gesture or
browser-initiated navigation.

The honest negative, which is itself a finding: **there is no standard for "the
readable article."** Readability, trafilatura and Postlight are all de-facto.
fleece's `Article` shape is correctly proprietary, and saying so on the record
stops the question being re-asked.

Two boundary rulings worth recording:

- **Web Annotation *Protocol*: SKIP** while adopting the Data Model. The Protocol
  builds on Linked Data Platform, models containers as LDP Containers and
  requires HTTP server interactions — a server-tenancy design, flatly against
  serverless-by-default. Adopting the model without the protocol is coherent, but
  it has to be written down or someone will assume they travel together.
- **Selectors and States: SKIP.** It is a 2017 Working Group *Note*, not a
  Recommendation; the selector vocabulary that matters is already normative
  inside the Data Model.

### 3.2 tabard: DTCG went stable, and there is no Rust crate

**Design Tokens Format Module 2025.10**, published 2025-10-28, is the first
stable release: "This specification is considered stable. Further updates will be
provided in superseding specifications." tabard's README already calls DTCG the
non-negotiable interchange form; that call is now backed by a frozen target
rather than a moving one. The **Color Module** ships alongside it; the **Resolver
Module** is still a WATCH.

Conformance is thinner than it looks but has teeth: every token MUST carry an
explicit `$type` (inference from the value is *prohibited*), circular references
MUST be detected, and tools MUST preserve `$extensions` they do not themselves
understand — a round-trip that drops a foreign vendor's extension block is
non-conforming. There is no conformance test suite and no normative reference
implementation, so compliance is self-asserted against a date string.

The internal argument is stronger than the interchange one: if tabard's derived
palette **is** a DTCG token tree, then the CSS custom-property emitter for
livery, the theme-struct emitter for hosts, and any future vendor emitter all
become projections of one model instead of three parallel derivation paths.
`$extensions` under a reverse-DNS key is the sanctioned home for tinct's seed
colours and OKLCH ladder parameters, which is what lets a livery round-trip back
into the authoring surface.

**crates.io has no DTCG parser/emitter as of 2026-08.** All reference
implementations are JS/TS (Style Dictionary, Tokens Studio, Terrazzo). That is a
small, well-scoped, unclaimed job — naming-ledger territory, not decided here.

Two corrections for tabard specifically:

- **APCA: SKIP.** It was pulled from WCAG 3 in July 2023, and the current WCAG
  3.0 Working Draft (3 March 2026) names no contrast algorithm at all — it refers
  only to an unspecified "minimum contrast ratio test". Build contrast-gated text
  roles to **WCAG 2.2 SC 1.4.3 / 1.4.6 / 1.4.11**.
- **CSS Custom Properties Level 1** is the normative definition of tabard's own
  *output* and was missing from the survey. It defines the custom-property
  syntax, the `var()` fallback grammar, and the guaranteed-invalid value that
  governs what happens when a token fails to resolve. It has sat at Candidate
  Recommendation since 2022.

Underneath both: **sRGB is IEC 61966-2-1:1999**, paywalled, which is why
implementations are written from the CSS Color 4 restatement instead. Its
piecewise transfer function (0.04045 / 12.92 / 2.4) is routinely mis-implemented
as a plain 2.2 gamma — exactly the error class that silently breaks contrast
gating.

### 3.3 pelt: mostly unglamorous, with three that matter

The bulk is desktop integration: XDG Base Directory 0.8, Desktop Entry 1.5, XDG
Desktop Portals (Settings/`org.freedesktop.appearance`, FileChooser, Screenshot,
OpenURI) — all ADOPT, all boring, all cheap. shared-mime-info and the Icon Theme
spec are PULL; the Windows/macOS registration equivalents are PULL and are not
standards at all.

Three worth attention:

- **Core-AAM 1.2 is meaningless without WAI-ARIA 1.2** as its source vocabulary.
  Core-AAM is a mapping table. Adopting one without the other is incoherent, and
  the survey did exactly that until the fact-check caught it. ARIA 1.3 remains a
  Working Draft (4 June 2026) and has not reached CR, so 1.2 is the build target.
  Core-AAM 1.2 is now a Candidate Recommendation Draft of 5 August 2026; AccName
  1.2 is a Working Draft of the same date.
- **Global Privacy Control** has one operative California date: **2027-01-01**
  (AB 566). The business-side duty to honour an opt-out signal comes from
  existing CCPA/CPRA regulation, not from AB 566, and conflating them misstates
  what is required and when. The spec surfaces are all confirmed: `Sec-GPC` MUST
  be exactly `1` when enabled and MUST NOT be sent when disabled; intermediaries
  MUST NOT strip it; `navigator.globalPrivacyControl`; `/.well-known/gpc.json`.
- **ITU-T H.273** is the shared code-point registry that PNG's `cICP`, the
  Wayland colour-management protocol, ICC's CICP tag and AV1/HEVC all point at.
  It is the one document that stops colour identity being re-derived at every
  boundary between an image decoder, a compositor protocol and an ICC profile.
  Relevant to netrender and wgpu-scry as much as to pelt.

Wayland colour-management (`color-management-v1`) is Staging, interface version
2, first shipped in wayland-protocols 1.41. Version 2 **deprecates the `srgb` and
`ext_srgb` named transfer functions** in favour of `compound_power_2_4` — an
implementation written to the older enum list targets deprecated values.

---

## 4. The largest hole is in none of them: local-link discovery

**Corrected 2026-09-01.** As first written, this section claimed no local-link
discovery story existed anywhere in the stack. Half right. Peer discovery over
mDNS is in production: p2panda's mDNS runs `Active` in Turnstone, Graphshell,
Knot and Djinn behind Murm's `P2pandaOverlayHost`, with two branch-pinned forks
and a macOS signing finding. The home for that record is R0 of the
[reachability rungs plan](mere_docs/implementation_strategy/2026-08-03_reachability_rungs_and_privacy_lanes_plan.md).
What is absent is DNS-SD service browsing and advertisement (RFC 6763), planned
work owned by Murm with Djinn's registry (F3) and the reference host plan's H10
as consumer. The rest of this section stands for that half. The survey covers
global directory (WebFinger, JSContact, DID, NIP-05) and long-haul radio
(Reticulum, LoRa), and between them sits only the peer-discovery rung.

**RFC 6762 (mDNS) + RFC 6763 (DNS-SD)**, both Proposed Standard, are the layer
that distillery's device ring, moot's places, and turnstone's shared addressable
places all need: peers on a LAN with no accounts and no cloud tenancy. It is the
discovery layer a serverless house posture actually lives on, and `mdns-sd` is
pure Rust. Gotcha: Windows and macOS hold port 5353, so an in-process responder
collides with the platform resolver; "Bonjour" is Apple's trademark for the same
protocol.

**Verdict: ADOPT**, and it deserves its own plan rather than being absorbed into
a port. (2026-09-01: for peer discovery that plan is the reachability rungs
plan's R0; DNS-SD service browsing is still to be planned, in Murm.)

Three more cross-cutting items:

- **RFC 8615 (Well-Known URIs)** is the substrate under `/.well-known/webfinger`,
  `/.well-known/nostr.json`, `/.well-known/did.json`,
  `/.well-known/change-password` and `/.well-known/gpc.json` — five consumers
  already named in this brief. Its IANA registration requirement (a name must
  reference a spec defining format and media type) is a design constraint on any
  house-invented well-known path.
- **RFC 9309 (Robots Exclusion Protocol)** has been Standards Track since 2022,
  contrary to the widespread belief that it is convention only. fleece extracts
  from fetched pages and turnstone fetches them; anything retrieving third-party
  content at more than one page at a time is a crawler by the RFC's own
  definition. Four normative pages, cheap to build to, expensive to be caught not
  building to.
- **RFC 5005 (Feed Paging and Archiving)**, Proposed Standard, is the only
  standard that says how a feed exposes its whole history — `fh:complete` and the
  `prev-archive`/`next-archive` link relations. Atom and RSS describe only a
  current window. Without it, gazette and turnstone each invent a paging
  convention.

---

## 5. The other ports, briefly

- **gazette.** **RFC 9553 JSContact** (with RFC 9554/9555 for vCard conversion)
  provides an exchange format for Gaz, which keeps its own model at rest
  (Mark's 2026-09-23 ruling). Optional Gaz exchange landed 2026-09-30 with
  distinct public cards, unverified peer imports and private restoration;
  vCard conversion and Gazette application wiring remain open. ADOPT over
  vCard 4.0's PULL. WebFinger (RFC 7033)
  stays ADOPT. **Verifiable Credentials 2.0** is the standard the house "insigne"
  concept most closely already is — graded public-key presentations made to be
  shown — and is worth reading before insigne's own shape is fixed. **DIDs,
  amended 2026-09-23 by Mark's ruling, and graded per method rather than as one
  family.** The first pass carried `did:key` and `did:web` as "the only two
  methods worth carrying" and recorded no reason. Judged on what a contact
  anchor needs (stable per account, able to rotate keys, bound by something
  other than a host), it kept the weakest method and dropped the only one that
  meets all three. `did:key` is **ADOPT** as the text form of a typed key, which
  is how gaz writes every key; it can never rotate, so as an identity it is only
  a key. `did:plc` is **ADOPT** for gaz anchors and gazette resolution: stable
  per account, rotating, and self-certifying (see the atproto bullet below).
  `did:web` is **PULL** and is carried as a handle only, because it is a
  domain's say-so with no history.
  CardDAV and JMAP: SKIP, both server-tenancy. **Nostr NIP-01 is the load-bearing
  document, not NIP-05** — NIP-05 maps a name to a hex pubkey and is meaningless
  without the event/relay/signature model NIP-01 defines.
- **atproto, identity layer — checked 2026-09-23 against live data.** Sources:
  the PLC documents and `/log/audit` histories of
  `did:plc:z72i7hdynmk6r22z27h6tvur` (`bsky.app`) and
  `did:plc:ewvi7nxzyoun6zhxrhs64oiz` (`atproto.com`), and two 1,000-operation
  samples of `https://plc.directory/export` (from 2023-04-01 and 2026-08-01).
  - *Self-certifying, verified here rather than quoted:* each DID re-derives
    exactly as base32 of SHA-256 over the DAG-CBOR genesis operation, truncated
    to 24 characters, and a tampered copy of the operation does not. What it
    binds is that operation's rotation keys, and for both accounts those are
    the same two keys, held by Bluesky: the DID ties the account to whoever
    holds its rotation keys. The directory can withhold operations or show
    different clients different histories; it cannot forge one.
  - *Signing keys belong to the server, not the person:* both accounts were
    created with the same signing key, and all 895 accounts in the 2023 sample
    shared two keys (746 on one). The 2026 sample has one key per account, but
    the protocol does not require it. Keys change on every server move with no
    signature from the old key; the authority is a PLC operation signed by a
    rotation key.
  - *Key types:* secp256k1 (`zQ3s…`) and P-256 (`zDn…`), 33 bytes compressed;
    the 2026 sample is 889 P-256 to 111 secp256k1.
  - *Not a moving target:* every operation in both samples is `plc_operation`
    with the identical seven-field shape, three and a half years apart. The
    WATCH grade stands for atproto's repository and sync layers; the identity
    layer (`did:plc` and handle resolution) is **ADOPT** for gaz and gazette.
  - Consequences for gaz (anchoring, collisions, proofs of key changes) live in
    the gaz founding plan (`repos/dramatis/design_docs/2026-08-08_gaz_founding_plan.md`)
    §2 and §5.
- **moot / murmur.** MLS (RFC 9420) is ADOPT *here* even though it is SKIP for
  the vault — group messaging is what it was designed for. MIMI is WATCH. Matrix:
  SKIP (and note the survey's v1.16 citation was three releases stale; current is
  v1.19, 8 July 2026). Signal's published specs are XEdDSA, X3DH, PQXDH, Double
  Ratchet, Sesame and **The ML-KEM Braid** — "SPQR" and "Triple Ratchet" are
  announcement vocabulary with no citable document. **XMPP (RFC 6120/6121)
  deserves an explicit SKIP on the record** rather than silent omission: it is the
  only IETF Standards Track federated messaging suite, and RFC 6121's
  roster/presence model is prior art that gaz and moot are re-deciding from
  scratch.
- **signalman / retinue.** The binding standards are **regulatory, not
  protocol**, and they are hard constraints: **FCC Part 15.247** (US,
  902–928 MHz), **ETSI EN 300 220-2 V3.3.1** (EU SRD duty cycle, harmonised under
  RED), **ARIB STD-T108** (Japan). ADOPT. FCC Part 97 §97.113(a)(4) prohibits
  encrypted transmissions in the amateur service, which is precisely why
  unlicensed ISM is the floor rather than amateur allocation — that is a SKIP with
  a reason, not an oversight. LoRaWAN: SKIP (a network-server architecture,
  against the house posture). Reticulum RNS and LXMF are single-implementation
  specs, not standards — worth recording so the dependency is understood as such.
  **Checked 2026-09-23 against that implementation**, for gaz's Reticulum key:
  a public identity is 64 bytes, "the concatenation of a 256 bit encryption key,
  and a 256 bit signing key" (X25519 then Ed25519; `RNS/Identity.py`,
  `get_public_key`), and its identity hash is a truncated SHA-256 over all 64
  (`update_hashes`). `rnid` imports and exports a public identity as
  undelimited lowercase hex by default, base32 or base64 on request
  (`RNS/Utilities/rnid.py`, `export_pub_identity`). prns's
  `PublicIdentityMaterial` (workspace-root checkout, `Code/crates/prns/prns-core/src/identity/material.rs`)
  agrees on the layout.
  **IEEE 802.15.4-2024 with RPL/6LoWPAN** deserves a named SKIP so retinue's
  bespoke mesh is a deliberate choice rather than an unexamined one.
- **distillery / alembic.** safetensors: ADOPT. GGUF: PULL (current format
  version is 3; v2's change was uint32→uint64 counts, not alignment padding).
  MCP: WATCH. ONNX and OCI model packaging: SKIP. **There is no standard for
  job/lease/heartbeat semantics in a P2P device ring** — Kubernetes Lease,
  Sparkplug B, Redfish and DRMAA were all surveyed and rejected. Staying
  proprietary is correct and is now on the record. **EU AI Act Article 50**
  transparency obligations are worth a read for whether a local-inference product
  is in scope.
- **knot / import.** CommonMark 0.31.2 is ADOPT; **Djot has no versioned
  specification** and its reference implementation is at 0.2.0 — WATCH, and note
  that **there is no registered media type for Djot at all**. RFC 7763 registers
  `text/markdown` with a `variant` parameter and RFC 7764 registers CommonMark as
  a variant; a knot projection or smolweb response serving Djot must pick
  something unregistered and live with it. That should be a recorded decision.
  The Netscape bookmark format `import` parses has no specification whatsoever.

---

## 6. The apps

- **woodshed / hocket.** MusicXML 4.0 is the practical notation target (Final
  Community Group Report, **1 June 2021**). MNX has not stabilised; MEI 5.1
  (released 2025-01-22) has the strongest tablature story via its tablature
  module but is SKIP on cost. MIDI 1.0 is ADOPT and is not deprecated; MIDI 2.0
  is WATCH, and note the actual SMF successor is the unfinished **SMF2 Container
  Format**, not the already-published MIDI Clip File. **ITU-R BS.1770-5 with EBU
  R 128** is ADOPT and is the thing a practice tool must not get wrong. **RFC
  9559 (Matroska, October 2024)** belongs beside RFC 9639 — the same working
  group gave both change control in the same quarter, and Matroska is the
  container a loop recorder writes multi-track takes into. LV2 is the third
  plugin ABI and the only one with no licence question in its history. And the
  cheapest adoption in the whole survey: **ISO 16:1975** (A4 = 440 Hz). Verified
  2026-08-24: it currently lives in `woodshed-audio/src/input.rs:122` and `:125`
  as a bare `440.0` inside the pitch/MIDI conversion, unnamed and unattributed,
  which is also the place a configurable reference pitch would have to enter.
  (The ISO designator itself could not be confirmed against iso.org, which
  returns 403 to automated fetch — check the current edition by hand.)
- **wavicle / pipit.** **FLAC is now RFC 9639** (Proposed Standard, December
  2024) — a genuine change from wiki-spec to change-controlled document, and
  Vorbis comments are normatively defined in its §8.6. For pipit: **PESQ (ITU-T
  P.862) was withdrawn on 2024-01-05**, so any quality claim must use P.863 POLQA
  or ViSQOL. ReplayGain 2.0 lives only on a wiki that 403s automated readers — no
  change control, which strengthens the PULL.
- **sonance / mora.** BCP 47 is ADOPT. **SSML is SKIP** — a 2010 Recommendation
  aimed at synthesiser control, not prosodic analysis — but its lexicon half, the
  **W3C Pronunciation Lexicon Specification (PLS) 1.0**, is exactly sonance's and
  mora's domain: a phoneme/alias lexicon keyed by alphabet identifier and BCP 47
  tag. Covering SSML alone leaves the pronunciation store unstandardised, which
  is where the data actually lives.
- **turquet.** The astronomy standards are unusually binding, and one of them is
  about to move. **Continuous UTC is pending a vote at the 28th CGPM, 13–15
  October 2026.** Draft Resolution C (version 5, 13 July 2026) "decides that
  continuous UTC will become effective on 20 May 2027" with |UT1−UTC| capped at
  3 600 seconds — a dramatic acceleration of the 2022 Resolution 4 language,
  which said only "in, or before, 2035". If adopted, UTC−TAI becomes a constant
  after that date: leap seconds stop, conversion becomes arithmetic rather than a
  table lookup, and turquet loses a live data dependency, which matters for a
  crate whose design goal is self-containedness. The countervailing risk is in
  the draft's own *noting* section: a March 2025 CCTF/IERS workshop put the
  probability of a **negative** leap second at 30% by 2035, one has never been
  applied, and essentially no implementation in the wild handles one. **WATCH,
  with a real date.**
  On reference frames, the survey named only half of what turquet needs.
  **ICRF3** (IAU 2018 Resolution B2) is the *radio* realisation of the ICRS —
  4536 VLBI quasars. turquet plots stars in the optical, which live on
  **Gaia-CRF3**, the fundamental optical realisation per **IAU 2021 Resolution
  B3**, effective 2022-01-01. Code that treats "ICRS" as one thing and reaches
  for ICRF3 cites the wrong realisation for the data it is drawing. Both ADOPT.
  Missing between the frame and the ephemeris is the *transformation* standard:
  **IAU SOFA** with **IERS Conventions (2010), Technical Note 36** is the
  IAU-endorsed chain from ICRS to apparent place, and nothing computes an
  altitude and azimuth without it. Gotcha that makes this a verdict rather than a
  footnote: SOFA ships C and Fortran 77 only under its own bespoke licence —
  neither MPL-2.0 nor MIT/Apache-2.0 — so it is a specification to reimplement
  against and validate with, never a dependency to link. NAIF SPICE and the JPL
  DE ephemerides stay ADOPT for interchange even though the ephemeris choice
  itself is settled: the analytic provider matches DE440s at millidegree
  precision and DE440s survives as an opt-in oracle rather than a runtime
  dependency, per `turquet/design_docs/2026-08-13_provider_architecture.md`.
  FITS/WCS, VOTable and SAMP are PULL. Do **not** quote a fixed total for the IAU
  Catalog of Star Names in any doc — it is continually updated, the survey's
  figure of 605 could not be corroborated against any primary source, and the 59
  names it attributed to 2024–25 were actually adopted and announced on
  2026-02-12. Read the count off the catalogue at time of use.
- **smolweb.** The finding is the taxonomy itself: **gopher (RFC 1436), finger
  (RFC 1288) and DICT (RFC 2229) are real RFCs; Gemini has a maintained
  two-document specification (protocol v0.24.1, gemtext separately since the
  March 2024 refactor); and nine protocols — nex, spartan, guppy, scroll, text,
  scorpion, kepler, misfin, fsp — have no standard designator of any kind**, only
  a reference implementation or a text file at an author's URL. For a repo whose
  stated purpose is spec-faithful implementation, knowing which half has a spec
  to be faithful *to* is the load-bearing fact. Two refinements: the datatracker
  now stamps RFC 1436 and RFC 2229 "Legacy — not endorsed by the IETF and has no
  formal standing", which strengthens rather than weakens that reading; and
  RFC 1288 sits at a maturity level RFC 6410 abolished outright.
  **Actionable, verified in code 2026-08-24: RFC 8446 is obsolete.**
  `draft-ietf-tls-rfc8446bis` was published as **RFC 9846** (Proposed Standard,
  July 2026), which obsoletes RFC 8446 along with RFC 5077, 5246, 6961, 7627 and
  8422 — it keeps the same wire version and is backward compatible, but the
  citable document for the TLS 1.3 that Gemini mandates is now RFC 9846.
  `smolweb/crates/scorpion-protocol/src/tls.rs:29` currently cites "RFC 8446
  §C.4" in a doc comment. That is a one-line fix in a repo outside this brief's
  scope; flagged, not made.
- **iconvg, isometry, the games.** IconVG is one person's format with an
  Apache-2.0 spec file, correctly pinned by the house decoder to a spec commit.
  Tiled TMX is the only virtual-tabletop-adjacent format with a real
  specification; "Universal VTT" has no document number at all. Lottie is at
  **1.0.1**, not 1.0 — a SKIP either way, but the point-release drift is the kind
  the "active but young" reading rests on. Games are a standards-poor area where
  proprietary formats are correct, and glTF is a SKIP absent a consumer.
  Two raster/texture omissions the survey missed entirely, both load-bearing:
  **PNG Specification (Third Edition)** reached W3C Recommendation on 2025-06-24
  and finally specifies APNG normatively, adds HDR through `cICP` with BT.2100
  PQ/HLG, and adds the `mDCV`, `cLLI` and `eXIf` chunks. It matters three ways
  here — isometry's entire asset vocabulary is PNG, tabard/tinct colour has to
  survive a round trip through PNG's colour chunks, and genet/pelt decode PNG on
  every page. Its `cICP` chunk is the concrete bridge between the BT.2100 and CSS
  Color 4 entries, and it is the same ITU-T H.273 registry named in §3.3.
  **Khronos KTX 2.0** is the GPU texture container with Basis Universal
  supercompression, consumed by netrender/wgpu-graft/wgpu-scry and the target of
  glTF's own `KHR_texture_basisu` extension — so the glTF entry had a dangling
  dependency. Likely PULL rather than ADOPT: the UASTC/ETC1S transcoders are the
  encumbrance-sensitive part and pure-Rust coverage is partial.

---

## 7. Verdict index

The survey returned 174 entries: sixty ADOPT, fifty-nine PULL, thirty-three
SKIP and twenty-two WATCH. The table held 175 rows, 23 of them WATCH.

**The verdicts now live in the family spec registry**,
`merelyllc.com/content/specs.toml`, whose rules are in
`merelyllc.com/docs/2026-10-18_family_spec_registry_plan.md` (phase R2,
2026-10-18). Each row became one or more claims: a spec, the repository and
crate that consume it, the verdict, and the row's consumer text as evidence. Each
claim is tagged `source = "mere-survey-2026-08-24#7.<row>"`, counting rows from
0 in table order. Bundled rows were split into one spec each. Where two rows grade
the same spec for the same consumer, the later verification wins.

The fact-checking pass corrected thirty-two entries after the fact, and the
corrections are recorded in §1, §3.3, §5, §6 and §8. Where a registry claim
conflicts with the prose above, **the prose wins**: correct the claim. PULL
rows were re-read in §7.1 under the 2026-09-23 definition, and the registry
carries that re-read.

---

## 7.1 PULL re-read under the 2026-09-23 definition

Mark redefined PULL on 2026-09-23, from "implement when a real consumer demands
it" to "design for it now, implement in order" (see the verdict grammar at the
top). The first meaning let a standard with a foreseeable consumer go undesigned
until the consumer arrived; the new one asks each PULL row a single question:
which data model, owned by a named consumer, reserves room for this standard
today? A row with an answer stays PULL and records it. A row whose
standard is already built moves to ADOPT. A row with nothing to design it into
(a hypothetical build, an "only if", a doctrine against it) moves to SKIP,
with what would reopen it. This section wins over the rows in §7 where they
disagree.

**Basis.** Each row's own consumer text, which §7's table truncated in places.
Where a grade turned on a fact, the fact was checked on 2026-09-23: which
crates depend on a UAX #29 segmenter, whether `murm/native/crates/webrtc-carrier`
exists, whether genet's design docs mention WebAuthn (they do not), and what
`repos/emblem` implements. The design-for notes name where room is reserved;
none of them is built by this re-read.

**Result.** Of fifty-nine rows, forty-two stay PULL, four move to ADOPT
because they are already built, ten move to SKIP, and three split.

Each re-read now sits on the registry claims tagged `+7.1` (see §7): the
design-for note is the claim's `evidence`, the grade is its `verdict`, and
`verified_on` is 2026-09-23. A split grade was applied to each part of the
bundle it named.

## 8. Method, and what it is not good for

Seven parallel survey lanes with live web access, each followed by an
adversarial fact-checker instructed to check only against primary sources
(ietf.org datatracker, w3.org/TR, nist.gov, fidoalliance.org, freedesktop.org,
khronos.org, ivoa.net, midi.org and each standard's canonical site) and to
default to suspicion. Fourteen agents, ~2.2M tokens, ~1 220 tool calls across
the initial run and one re-run.

**The fact-checking pass corrected thirty-two of the 174 entries.** That number
is the main methodological result, and it is why this brief carries verification
dates rather than bare statuses. A representative sample of what it caught: a
WebAuthn Proposed Recommendation that does not exist (there is no PR document;
W3C Process 2023 advances a CR Snapshot straight to REC); the wrong GitHub org
for the Public Suffix List; an atproto IETF working group whose datatracker page
reads "Abandoned" because the real group is `atp`, not `atproto`, and a cited
architecture draft that expired at revision -00; a Signal specification named
"SPQR" that has no such document (the citable name is *The ML-KEM Braid*); a
Matrix spec three releases stale; a California statute given an operative date it
does not have; MEI 5.1 misdated by a full year; a MusicXML URL that had moved
twice; and an ICC "orphaned from ISO" gotcha that ISO 15076-1:2025 had dissolved
ten months earlier.

**The sharpest finding is about confidence, not about any one standard.** Two
entries were self-flagged by their surveyor as recalled from training data rather
than re-fetched, on the reasoning that a long-settled specification does not
move. Both had moved, and both mattered: TLS 1.3's RFC 8446 was obsoleted by
RFC 9846 in July 2026, and WebRTC's Recommendation is a 13 March 2025 document
carrying substantive amendments rather than the finished 2021 one. The belief
that a settled standard is safe to quote from memory is precisely what produced
both errors. Treat that as the standing rule for this document's successors.

**Sources that could not be reached, so are recorded as unconfirmed:**

| Source | Blocked | Claim left unverified |
|---|---|---|
| iso.org | HTTP 403 to automated fetch | ISO 16:1975 current edition; ISO catalogue entries generally |
| trustedcomputinggroup.org | HTTP 403 on the listing page | Current TPM 2.0 Library revision (1.85 is search-index evidence, not a page read) |
| registry.khronos.org | HTTP 403 | SPIR-V Revision 7's 2026-03-12 date (the revision number itself is confirmed) |
| wiki.hydrogenaudio.org | HTTP 403 | ReplayGain 2.0's −18 LUFS reference and storage conventions |

Re-pin each of those by hand before any of them enters a plan.

**Three further limits.** The consumer column in §7 is the survey's assignment,
not a ruling — several entries guess at which crate would own a standard, and
those guesses have not been checked against the code. The IAU Catalog of Star
Names has no stable total and must be read off the catalogue at time of use;
do not quote a figure. And when citing DTCG, use the pinned
`designtokens.org/tr/2025.10/` rather than the rolling draft URL, or the version
claim silently drifts to whatever is current.


**Second pass, same day: a code-level verification of this document's own
claims.** Three further agents were run against the tree after the first draft
was committed, and they corrected this brief rather than the world:

- **§2.1's framing was wrong on cost and on coupling**, and is now revised in
  place with a banner. The original called the fix "a v2-root decision, not a
  patch"; a pass over the actual call graph found the two halves have nearly
  disjoint blast radii, no passphrase-ladder data exists on disk, and the
  read-old/write-new pattern is already house style in
  `sealed_record_storage.rs`. The claim was repeated from the survey without
  being tested. Testing it made the work cheaper, not more expensive — which is
  the argument for testing framing claims rather than inheriting them.
- **The §7 Consumer column was truncated mid-sentence in 151 of 174 rows**, an
  artifact of how the table was generated rather than of the survey. Regenerated
  at word boundaries. Anyone re-generating it should check that first — and
  should re-apply the hand corrections afterwards. Doing so once silently
  reverted four designator fixes (TLS 1.3, Lottie, WebRTC, ICC/ISO) already made
  by hand, leaving the prose asserting a correction the table no longer carried.
  A generated table and hand-edited rows do not compose: the corrections belong
  in the generator, or in a diff applied after it, never only in the output.
- **Five ADOPT rows named the wrong consumer** and are corrected in place: MLS
  pointed at `ports/moot` (a 43-line reservation that explicitly disclaims the
  role), VC 2.0 and RFC 5869 HKDF were listed against reservations and candidate
  future sites as though they were shipped dependencies, CSS Color 4 named
  `genet-livery` (an integration wrapper with zero colour code) where the same
  table elsewhere correctly names `livery`, and OpenType/COLRv1 cited `swash`
  where netrender uses `skrifa`.

- **Hand-edited after the generator, 2026-09-01:** one WATCH row added (Nym
  mixnet, beside the Reticulum row) and §4's opening claim corrected against
  the code. Both are edits to the output, exactly what the bullet above warns
  about: anyone re-generating §7 must re-apply the Nym row.

The generalisation worth keeping: the survey's *standards* research held up well
under adversarial checking, but its claims about **this codebase** — which crate
owns what, how expensive a change is — were the weakest part and needed a
separate pass with the code open. A standards brief written from READMEs will be
right about the world and wrong about home.

A note on the sweep instrument, because it nearly produced a false clean bill:
recursive `grep` over this workspace silently times out mid-traversal and returns
empty, which reads exactly like "no matches". The reliable instrument is
`git grep` per repo (it uses the index and skips `target/`). Any negative result
in a workspace-wide sweep should be accompanied by a positive control in the same
run, or it means nothing.
This brief should be re-checked against primary sources before it is used to
justify work, and certainly before 2027 — §1 exists because the last inventory
went stale in nine months.

---

## 9. Decisions this leaves open

Recorded here rather than resolved, because each has more than one defensible
answer and the choice is Mark's.

1. **The vault format and its parameters (§2.1).** Revised: this is *not* one
   decision with `derive_child`, and it is not a v2 root. It is a self-contained
   change to one crate with no data in the wild and an in-repo read-old/write-new
   precedent. The open forks are: which format shape — explicit `m_cost`/`t_cost`/
   `p_cost` fields with `FILE_VERSION` bumped to 2, a PHC string descriptor, or a
   shared `UnlockProfile` type that makes the "one unlock ladder" rule a type
   invariant rather than a comment? And which parameter target — RFC 9106's second
   recommended option (m=64 MiB, t=3) at ~5.1x current cost, or a lower number
   recorded explicitly as a deviation? The first recommended option (2 GiB) is
   unallocatable on wasm and should be ruled out on the record rather than left
   open.
2. **Should the manifest enforce the `argon2` hold?** `crates/dramatis/personae/
   Cargo.toml:44` reads `argon2 = "0.5"`; the 0.5.3 pin lives only in
   `Cargo.lock`. The crypto stack decision says the crate is held because a
   password-hash change is a stored-format change — but nothing stops a
   `cargo update` from moving it to 0.6, which changes the default parameters.
3. **The derivation context (§2.1), on its own timeline.** BLAKE3 `derive_key`
   (already a dependency, no RFC) or RFC 5869 HKDF (an RFC, but adding it to
   personae re-opens the digest-row question the crypto stack decision closed)?
   And are you willing to pay its real costs: a rollout where every peer ships a
   v2-aware verifier before any peer mints a v2 key, plus manual re-enrollment of
   the SSH CA on every host that holds it?
4. **Does the `passphrase_root` ladder have a future?** It has no consumers today
   and `StartupUnlockMode::Prompt` is unimplemented (`startup_unlock.rs:29-33`).
   If it is the non-Windows story, it belongs in scope for everything above. If
   it is speculative, deleting it shrinks the whole decision.
5. **CXF import policy (§2.3).** CXF v1.0 defines 17 credential types including
   `Passport`, `DriversLicense` and `CreditCard`. Which does castellan store,
   which does it drop on the floor, and which does it quarantine and tell the
   user about? This is a product decision that blocks the parser, not a
   consequence of it. **Ruled 2026-10-01** (Mark, rulings 10 to 15 in the
   dramatis tier architecture (`repos/dramatis/design_docs/2026-09-30_dramatis_tier_architecture.md`)
   §7): the everyday credentials, notes, custom fields, addresses and names
   are stored as chatelaine kinds; SSH keys go through castellan's SSH import;
   item references become links. Identity documents, payment cards, passkeys,
   files and types newer than v1.0 are quarantined: sealed, never exercised,
   accepted one at a time. Nothing is dropped.
6. **Whether a `dtcg` crate gets founded (§3.2).** There is no Rust
   implementation of a now-stable W3C-CG format that tabard needs anyway. That
   is either a small well-scoped piece of leverage or a distraction from tabard
   itself. Naming-ledger territory if the answer is yes.
7. **Where DNS-SD lands (§4).** Narrowed 2026-09-01: peer discovery landed in
   Murm (reachability plan R0), so the hole is service browsing alone, and the
   Djinn plan's F3 has since assigned it to Murm. As first written it had no
   obvious owner: distillery's ring, moot's places and turnstone's shared
   places all need it, which is an argument for a shared crate rather than three
   implementations, and an argument that it belongs to none of them.
8. **Insigne versus Verifiable Credentials 2.0 (§5).** VC 2.0 is close to what
   insigne already is. Adopting it buys interoperability and costs a large
   specification surface plus a JSON-LD dependency; staying proprietary keeps
   insigne small. Worth deciding before insigne's shape is fixed rather than after.
9. **Djot's wire identity (§5).** There is no registered media type. Serve
   `text/x-djot` and accept an unregistered name, serve rendered HTML, or pursue
   registration. Any of the three is defensible; drifting into one is not.
10. **One out-of-scope one-liner.** `smolweb/crates/scorpion-protocol/src/tls.rs:29`
   cites RFC 8446, now obsoleted by RFC 9846. Flagged, not fixed — smolweb was
   outside this pass's scope.

---

## 10. Transport composition and a shared conformance corpus — 2026-10-11

**Status:** owner-authorized direction; catalog and executable suite foundation
implemented and validated locally; retained-repair and bounded signed-object
owner receive prototypes implemented, including a real local QUIC handoff;
later integration and qualification open. Mark prefers published standards for covered behavior and asked
for orchestration, the useful specification scope, prior art and a WPT-like
testing set. This section is the canonical cross-repository research record.
Implementation sequencing lives in
`murm/design_docs/2026-10-11_transport_conformance_plan.md`.

**Execution posture, owner clarification:** prototype the composed stack toward
its final shape using code and canonical documents as the execution frontier.
Small local checks and design experiments support that work. Broad conformance
and regression automation follow stable behavior/contracts; they are not a
requirement for every interim state. Publish directly to `main`. GitHub Actions
is disabled in Mere/Retinue; this work adds no workflows, PRs or issue ledger.

The reviewed transport inventory is
[transport_catalog.toml](../support/standards/transport_catalog.toml), with
[prior_art.toml](../support/standards/prior_art.toml) recording reusable corpora,
tools, licenses and independent peers. These are authored TOML under the
[data-formats ruling](2026-10-06_data_formats_brief.md); generated run receipts
are JSON. The catalog contains 71 specifications or specification families, grouped into
25 candidate profiles: 60 RFCs, seven project specifications, one IEEE family
and three vendor families. It includes current baselines, updates, older normative
references, guidance and alternatives. It is a scoped inventory, not the full
transitive closure of every selected RFC, the whole Internet stack, or an
implementation/compliance assertion. Select a profile before expanding its
normative dependencies. Metadata verification and full errata review are
separate fields; unreviewed errata must not be recorded as absent.

### Composition target

Preserve original signed object bytes and identity across delivery paths.
Stickleback owns replicated-space verification/retention/store mechanics;
domain policy owns admission; Murm supplies carriers; Tulle/radio-hand own
radio interfaces and device execution. A portable FEC leaf owns coding. It
does not become another replication or authorization authority.

The resulting delivery choices are existing QUIC/IP, compatible Reticulum,
an optional Outrider/Dash courier, and a separately specified multi-channel
object-repair profile. A fast PHY can carry bulk traffic while LoRa supplies
reachability/control when that deployment supports it. Hardware candidates do
not establish board support or measured throughput. Full Internet multicast,
DTN and constrained-IP interoperability are additional profiles selected for
real peers, not obligatory layers under every application.

Adopt standards-compatible implementations where available and qualify the
adapter; implementing a standard does not imply writing new cryptography or a
new QUIC implementation. Do not substitute BLAKE3 into Reticulum's specified
SHA-256/HKDF-SHA256 wire algorithms. New COSE manifests/receipts may wrap transfer
metadata while retaining the original operation's signature and identity.

The exact operation profile must be revisioned. At inspected Mere
`0e78fcffb140395a5c0d2f64a4dfda2b81751681`, the lock selects p2panda-core 0.7.1
from the patched family at `d8acabc96ecc59016e40edf4a47ed5eaf5360285`. Its
[pinned header source](https://github.com/mark-ik/p2panda/blob/d8acabc96ecc59016e40edf4a47ed5eaf5360285/p2panda-core/src/operation/header.rs)
uses CBOR and preserves extension representation. The upstream release page's
Postcard statement is not sufficient evidence to change this pinned wire
profile. Signed bytes and compatibility follow the actual selected source.

### What the standards do and what remains ours

RFC 6330 supplies RaptorQ coding and exact payload-ID/OTI semantics. A CBOR
manifest over raw radio is a named project profile around that codec.
[ALC RFC 5775 §2.2](https://www.rfc-editor.org/rfc/rfc5775.html#section-2.2)
requires support for WEBRC and congestion control over repair as well as source
packets. [FLUTE RFC 6726](https://www.rfc-editor.org/rfc/rfc6726.html) has FDT,
expiry, format and security requirements. A private manifest/schedule does not
establish either transport's conformance. NORM is a separate return-path
interoperability branch; its name does not apply to custom deficit feedback.

RaptorQ's [verified technical erratum 5548](https://www.rfc-editor.org/errata/eid5548)
corrects maximum transfer length to 942,574,504,275 bytes. A profile can set far
smaller resource limits. Four-octet payload IDs and twelve-octet combined OTI
need exact vectors and malformed-input guards. Codec self-roundtrips are useful
composition evidence; exact symbol comparisons and decoding between independent
implementations remain a separate qualification gate. Arbitrary K received
symbols are not an unconditional recovery promise.

Our retained-repair design must define generations, authenticated metadata,
symbol namespaces, replay behavior, decoder/retention limits, repair floors,
receiver sleep/restart, clock/tuning costs and expiry. An object signature
detects failed reconstruction but does not reject each poisoned symbol before
expensive decoding; the profile needs authenticated delivery and bounded work.
The long-run source rate intended for a receiver must remain below its useful
collection capacity, and retained generations must survive long enough for
catch-up. More radios help collection; repair coding does not overcome a
permanent single-radio capacity deficit.

BPv7/BPSec is an optional real DTN adapter. SCHC's LoRaWAN binding applies to
LoRaWAN, not any raw-LoRa bearer. CoAP/OSCORE/EDHOC is an optional constrained
application interface. MLS/HPKE is selected only against an actual security
contract and existing group-security ownership. FECFRAME/RLC and Tetrys are
live-flow alternatives/research comparisons, not RaptorQ dependencies.

Metadata checked live also confirms [TLS 1.3 RFC 9846](https://datatracker.ietf.org/doc/rfc9846/)
obsoletes RFC 8446. Published QUIC RFCs still cite their original RFC 8446
baseline; record current deltas explicitly without implying a new TLS wire
version. RFC 8032 and RFC 9180 are Informational IRTF specifications, and
RFC 9407 is Experimental. BLAKE3, Bao, p2panda and Reticulum are project
specifications; PHY and IEEE specifications retain their own classification.

### Prior art and evidence model

Borrow [WPT's specification-organized design](https://web-platform-tests.org/test-suite-design.html):
stable cases, implementation-independent requirements, explicit optional
capabilities and per-environment metadata. Use a common control/report layer
with specialized upstream vectors and external peers. The browser harness is
not required for a transport corpus.

Useful seeds include official BLAKE3 vectors, RFC 8032 examples and reviewed
Wycheproof negative cases; selected COSE WG examples; QUIC Interop Runner and
Quinn endpoints; TLS-Anvil/tlsfuzzer; OpenRQ or another independently qualified
RaptorQ peer; NRL NORM; and NASA DTNTools/ION/HDTN for a selected BPv7 adapter.
Retinue already has a stock-RNS process oracle. Its currently pinned requirements
are RNS 1.5.7/LXMF 1.2.0; old captured-vector versions remain historical evidence.
Prns is withdrawn as a trusted donor/reference under Retinue's existing policy.
The catalog records license and profile restrictions before any corpus import.

No broadly maintained independent RaptorQ vector corpus was found in this
bounded survey. This is a useful potential contribution after comparing actual
independent encoders, not proof that none exists. OpenRQ's supported domain is
restricted and its maintenance is old. OpenFEC's inspected codec list does not
make it a RaptorQ oracle. GPAC's ROUTE support does not prove full FLUTE support.
CBOR RFC 7049 example vectors alone do not qualify RFC 8949 deterministic
encoding. Detailed source/provenance is in the prior-art catalog.

Report requirement/vector conformance, independent interoperability, composition,
simulation, fuzz/resource behavior, performance and physical qualification
separately. Unsupported, skipped, timeout, infrastructure failure and inconclusive
results remain visible. Expected failure tracks debt without changing the
requirement. Performance records useful admitted bytes, completion latency,
repair amplification, CPU/memory, storage, airtime and energy with assumptions.
The end product is a profile/requirement matrix and reusable corpus, with a
benchmark companion; no single compliance score or invented certification.

### Source verification and next implementation

Current source inspected at Mere `0e78fcffb140395a5c0d2f64a4dfda2b81751681` and
Retinue `7aa87f2e9fdd973e4729e610d7130189dc22c0a0`. Tulle's heterogeneous sans-io
modem seam still uses LoRaParams; Murm's Reticulum carrier is bilateral and
default-off; JoinedSpace still uses p2panda Endpoint/Gossip/LogSync; the
OperationProcessor verifies then asks domain policy before store mutation.
No production RaptorQ/FLUTE/NORM implementation was found in these trees.
These are current source findings, not device or RF qualification.

[Dash's September 16 update](https://p2panda.org/2026/09/16/dash-router-sync-api.html)
places routing/networking/sync together behind an external event stream into
p2panda processing and says proof-of-concept hacking is next. Preserve that
adapter boundary and test its delivery behavior when selected. The Retinue plan
sequences the executable seed, independent codec peers, portable adapters,
retained-repair simulation, original-byte admission, embedded resource profiles
and physical multi-radio qualification. Later protocol branches retain their
own gates.

**2026-10-11 implementation continuation:** Retinue's isolated
`testing/transport-conformance` workspace now contains a selected independent
OpenRQ/Rust RaptorQ corpus: six objects, 264 exact source/repair symbol
comparisons and 24 live cross-decoding scenarios per direction. Its shared
profile is `Al=1, N=1`, with full T-byte symbols and explicit restoration of
OpenRQ's omitted final-source zero padding. The offline runner verifies imported
artifact hashes, replays the Rust decoder and checks retained Java observations
against current Rust packet bytes; an external-process campaign reruns both
implementations. Complete RFC 6330 conformance remains explicitly unsupported.
The corpus, provenance, reproduction guide and per-case receipts live with the
Retinue execution plan, rather than duplicating them in this research catalog.

**2026-10-11 prototype continuation:** the same isolated workspace now models
retained originals, authenticated generation manifests/symbols, fresh priority,
a repair floor, expiry and partial-state resume with actual RaptorQ coding.
Four editable design experiments compare one/four-radio finite catch-up,
zero/one-floor scheduling and sustained overload. The model counts actual
authored frame/signature bytes and uses ideal simultaneous transmitter lanes,
preconfigured channel/manifest cadence and a common slot clock. Findings and
the historical source-fingerprinted report live once in
`murm/design_docs/2026-10-11_retained_repair_simulation_profile.md` and
`retinue/testing/transport-conformance/results/retained-repair-exploration-20261011.json`.
The owner-local receive continuation below now exercises application admission;
the model does not establish live carrier integration or physical Mbps.

**2026-10-11 signed-object receive continuation:** the Mere-owned
[host example](../crates/mesh/mesh/examples/retained_receive/README.md) preserves
one original signed V2 job record through actual RaptorQ recovery and passes it
to the existing mesh/operation processor admission path. The receiver chooses
its expected mesh independently of the offered record. The selected
[source-bound report](../crates/mesh/mesh/examples/retained_receive/receive-exploration-20261011.json)
observes one durable operation and posted job across four-radio and late
one-radio arrival, then unchanged state after three invalid-inner-record
refusals and in-process store reopen. Exact record/header/body bytes and the
operation ID are preserved; the record digest and generation identities retain
their separate purposes. The example records codec, processor and domain
refusals without adding an admission owner. It transfers job metadata, not
referenced blobs or worker execution. The QUIC continuation below now composes
a real carrier; wider evaluation remains deferred until stable contracts.

**2026-10-11 real carrier continuation:** the same example now has an explicit
`--carrier quic` path using production `P2pandaTransport` over p2panda-net/iroh.
Stickleback's generic object-stream leaf preserves opaque bytes, caps declared
length before allocation and requires complete payload plus clean FIN within
one deadline. The receiver checks protocol and authenticated transport peer,
then freshly decodes the operation and calls the same mesh admission owner.
The chosen expected mesh and inner author remain separate from carrier facts.
A compact acknowledgement follows the owner's result and binds record digest
and operation ID; sender receipt is reported separately from local admission.

The [example guide](../crates/mesh/mesh/examples/retained_receive/README.md) owns
the bounds and reproduction command; its
[QUIC report](../crates/mesh/mesh/examples/retained_receive/quic-handoff-exploration-20261011.json)
owns the source/executable identity and observations. The finite host permits
one in-flight exchange, at most 12 attempted sessions, 16 KiB objects, 2 KiB
acknowledgements, 64 KiB of request frames and 128 KiB reserved request/reply
frames. Refusals count before dialing. It observes actual encrypted loopback
QUIC, unchanged signed bytes, insertion/duplicate/domain refusals, three
pre-owner framing refusals and valid subsequent handoffs. Loopback peer routes
and active paths are checked; endpoints retain production bind defaults.
Production global session admission, additional carriers and cross-host/device
integration remain later composition work. Modeled radio opportunities do not
acquire physical throughput evidence from this local IP handoff.

### MemoryLAN: adapt inventory policy and keep a baseline

[MemoryLAN](https://github.com/p2panda/memorylan/tree/f76039fc585b3f417e372167aa5825561fc9dd88)
combines whole-page fast flooding with probabilistic Cuckoo-filter summaries
for bounded hot-cache repair. Its useful place is an opportunistic inventory
and gossip policy alongside retained-object delivery. A summary can suggest an
object to request or schedule. It does not represent RaptorQ generation state,
decoder progress, symbol IDs or durable retention. Select **ADAPT** for the
policy and **BASELINE** for comparison; no production import is implied.

This assessment inspected upstream base
`f76039fc585b3f417e372167aa5825561fc9dd88`, local eviction-fix revision
`44610185aa074fe264df08b6de635766d1113c8e`, and local research revision
`493652cd980eab53a35920b9cb5d11c28f794c88`. The latter adds the harness and
evidence; both local commits are unsubmitted. The existing source-hashed host
report was reviewed rather than rerun in this transport pass. Its eight-node
ring has 512 synthetic pages, volatile caches and discrete ticks. Across the
ten-seed combined-fault experiment, mean historical remote delivery was 74.99%
with summary repair versus 46.34% with fast flooding alone, at roughly 4.9 times
the modeled bytes. Cache sensitivity was substantial: 16/64/256-page caches
delivered 59.34%/74.99%/97.37%. Extending one combined run from 1,024 to 8,192
ticks increased traffic without improving historical delivery; globally cold
restarting all switches left every cache empty. These findings support
hot-working-set repair and expose the need for a retained/durable layer.

The prototype uses hypothetical page/envelope byte costs and omits radio
contention, fragmentation, signatures, duty cycle, physical time and actual
operation admission. The ratios cannot be converted to radio Mbps. The local
eviction fix inserts an incoming page's fingerprint after removing the evicted
one; comparison must identify that patched behavior instead of claiming the
unmodified upstream revision produced these observations.

A downstream adapter should preserve original signed operation bytes and
canonical IDs, carry bounded envelopes over Reticulum, Dash or IP, and pass
recovered objects through the existing single admission owner. Required profile
choices are explicit versioned stable hashing of IDs and filter indices
(MemoryLAN uses Rust `DefaultHasher` internally), byte-budgeted caches and
replies, authenticated and aged/capped neighbor accounting, and fallback or
rebuild when filter insertion fails. Periodic exact inventory/durable
reconciliation can complement probabilistic summaries. Current cache/history
limits count items; full-page replies are not MTU/rate-limited, and filter
parameters lack negotiated wire identity. Those are adapter design questions,
not failures of the reported synthetic corpus. The source declares
`MIT OR Apache-2.0`; license and observed/evidence revisions are recorded in the
prior-art catalog. No maintainer contact or generated upstream submission is
part of this work.

**2026-10-12 inventory-policy continuation:** Stickleback now has an IO-free
[`inventory` leaf](https://github.com/merely-made/eidetic/tree/main/crates/stickleback/src/inventory/mod.rs) and a
[finite comparison harness](https://github.com/merely-made/eidetic/tree/main/crates/stickleback/examples/inventory_compare/README.md).
The retained owner announces whole-record digests, lengths and expiry; it keeps
payload storage and eviction. The policy has 64-entry/1 MiB/16 KiB ceilings,
eight aged peer slots, a 1 KiB control ceiling and eight bounded offers per
decision. Immutable snapshots support atomic exact pagination. Versioned
domain-separated BLAKE3 defines Cuckoo bucket/fingerprint hashing; summaries are
rebuilt, and saturation selects exact fallback instead of dropping fingerprints.
Offers rotate across decisions. Read permission, authenticated peer context,
payload retrieval and operation admission remain with the existing owners.

The [selected report](https://github.com/merely-made/eidetic/tree/main/crates/stickleback/examples/inventory_compare/results/20261012/comparison.json)
contains 105 finite host runs with byte-identical independent-process replay;
[source/toolchain provenance](https://github.com/merely-made/eidetic/tree/main/crates/stickleback/examples/inventory_compare/results/20261012/metadata.json)
and the [grouped summary](https://github.com/merely-made/eidetic/tree/main/crates/stickleback/examples/inventory_compare/results/20261012/summary.json)
retain exact assumptions and raw outcomes. Eager push is compared with eager
plus exact repair and eager plus Cuckoo/periodic exact fallback. The two shared
receiver opportunities per round, positive repair reservation, control framing,
continuation requests, payload attempts and reserved result replies are counted.
Fitting 32-object and eight-object recovery cases reach every peer in both repair
modes. A surviving retained source restores all 32 objects after global volatile
restart; without a source every final cache remains empty. Historical receipt
does not imply retention or post-restart recovery.

For the 256-object combined-fault churn, mean historical remote receipt is
47.61%/53.92%/55.18% for eager/exact/Cuckoo, at approximately
0.98/2.98/2.91 million modeled attempted frame bytes. In clean overload, the
repair reservation reduces receipt from eager's 73.44% to 59.54%/59.82%; Cuckoo
uses about half exact repair's modeled bytes in that case. The fitting case
recovers fully with either inventory mode, with exact using fewer modeled bytes.
The first live producer's four-object/64 KiB profile should begin with exact
inventory; choose Cuckoo against a concrete workload and budget. These are
scheduling/policy observations, not universal delivery guarantees or a pure
filter comparison independent of scheduling.

The shared resident-ingress proposal was coordinated with the Reticulum lane:
four queued sessions per ALPN, two active submissions across peers, 16 KiB
objects and 2 KiB result payloads, with host-owned complete-call deadlines.
That resident service/admission API remains a proposal. Inventory peer metadata
and pending object IDs are not session slots; live dispatch must reserve the
owning service's capacity. Inventory-read and submission permissions are
distinct. The future admitted service needs its own ALPN/handshake and an
explicit authorized inventory path. The existing bounded object-stream helper
is available today. This slice exercises synthetic digest-verified reception,
not sockets, signed operation admission, RaptorQ, RF, physical time or MCU
resources. The earlier MemoryLAN experiment remains a separate pinned baseline,
not a paired result with this new workload. Publication uses direct commits and
pushes with local checks; no workflows or PRs are added.

**2026-10-12 exact live composition continuation:** The
[admitted inventory/ingress service and local campaign](../crates/mesh/mesh/examples/inventory_ingress/README.md)
now implement that bounded host slice. Distillery mounts it on its existing
endpoint and mesh store. Notochord admits separate `inventory` and `submit`
actions; both share two active calls, four pending/queued reservations acquired
before first-stream waiting, and one arrival-to-response-FIN deadline. The
16 KiB object, 2 KiB result and 1 KiB exact-control limits are enforced alongside
whole-call actual-I/O ceilings. Original record bytes are freshly decoded and
authenticated before the existing `MeshStore::accept_outcome`; only that owner
changes durable domain state.

Retinue's live four-object/64 KiB producer supplies exact metadata and actual
repair-only RaptorQ decoder output. With one record already available at the
receiver, its admitted exact snapshot selects exactly the other three. The
[captured composition](../crates/mesh/mesh/examples/inventory_ingress/results/20261012/composition.json)
and [provenance](../crates/mesh/mesh/examples/inventory_ingress/results/20261012/metadata.json)
record four original/header/body preservation checks, unchanged operation IDs,
one durable effect per original, and a separate-process store reopen. Two
independent campaigns agree on delivery/admission semantics. Action mismatches,
invalid records and incomplete/oversized/trailing framing are refused; six
stalled calls fill the shared active/queue ceilings, a seventh is refused,
deadlines drain the work, and a healthy duplicate succeeds afterward.

The producer refuses a fifth live generation without eviction. The receiver's
volatile four-original availability window FIFO-evicts on a fifth accepted
original and expires bytes with its exact metadata, without deleting durable
operations. Resident mount/expiry/stalled-shutdown checks cover that ownership
boundary; restart begins with empty availability metadata. Producer logical
slots and receiver milliseconds are independent, and digest membership conveys
no cross-clock expiry or custody claim. Actual coded-frame delivery remains
host loopback before recovered originals cross real QUIC; physical radio, MCU,
worker execution and native UI are outside this receipt. The earlier finite
loss/partition/restart experiment remains separate evidence.
